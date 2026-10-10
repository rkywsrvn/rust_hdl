use super::analyze::*;
use super::scope::*;
use crate::ast::token_range::WithTokenSpan;
use crate::ast::*;
use crate::data::error_codes::ErrorCode;
use crate::data::*;
use crate::named_entity::*;
use vhdl_lang::TokenSpan;

/// Analysis of assignment targets
///
/// examples:
///   target <= 1;
///   target(0).elem := 1
impl<'a> AnalyzeContext<'a, '_> {
    pub fn resolve_target(
        &self,
        scope: &Scope<'a>,
        target: &mut WithTokenSpan<Target>,
        assignment_type: AssignmentType,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> EvalResult<TypeEnt<'a>> {
        match target.item {
            Target::Name(ref mut name) => {
                self.resolve_target_name(scope, name, target.span, assignment_type, diagnostics)
            }
            Target::Aggregate(ref mut assocs) => {
                self.analyze_aggregate(scope, assocs, diagnostics)?;
                Err(EvalError::Unknown)
            }
        }
    }

    pub fn resolve_target_name(
        &self,
        scope: &Scope<'a>,
        target: &mut Name,
        target_pos: TokenSpan,
        assignment_type: AssignmentType,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> EvalResult<TypeEnt<'a>> {
        let object_name = self.resolve_object_name(
            scope,
            target_pos,
            target,
            "may not be the target of an assignment",
            ErrorCode::MismatchedKinds,
            diagnostics,
        )?;
        if !object_name.base.can_be_assigned_to() {
            diagnostics.add(
                target_pos.pos(self.ctx),
                format!(
                    "{} may not be the target of an assignment",
                    object_name.base.describe_class()
                ),
                ErrorCode::MismatchedKinds,
            );
        } else if let Some(part) = self.view_part(target) {
            if !part.can_be_assigned_to() {
                diagnostics.add(
                    target_pos.pos(self.ctx),
                    part.describe_illegal_target(target),
                    ErrorCode::MismatchedKinds,
                );
            }
        } else if !object_name.base.is_valid_assignment_type(assignment_type) {
            diagnostics.add(
                target_pos.pos(self.ctx),
                format!(
                    "{} may not be the target of a {} assignment",
                    object_name.base.describe_class(),
                    assignment_type.to_str()
                ),
                ErrorCode::MismatchedKinds,
            );
        }
        Ok(object_name.type_mark())
    }

    /// Returns the part of a view port that `name` denotes,
    /// or `None` if `name` does not denote a part of a view port.
    fn view_part(&self, name: &Name) -> Option<ViewPart<'a>> {
        match name {
            Name::Designator(designator) => {
                let object = ObjectEnt::from_any(self.arena.get(designator.reference.get()?))?;
                let Some(InterfaceMode::View(view)) = object.mode() else {
                    return None;
                };
                if matches!(object.type_mark().base_type().kind(), Type::Array { .. })
                    && !matches!(view.subtype().base_type().kind(), Type::Array { .. })
                {
                    Some(ViewPart::Views(*view, false))
                } else {
                    Some(ViewPart::View(*view, false))
                }
            }
            Name::Selected(prefix, designator) => match self.view_part(&prefix.item)? {
                ViewPart::View(view, conversed) => {
                    let mode = view.view().element_mode(designator.designator())?;
                    Some(match mode.converse_if(conversed) {
                        ViewElementMode::Simple(mode) => ViewPart::Element(mode),
                        ViewElementMode::Record(view, conversed) => ViewPart::View(view, conversed),
                        ViewElementMode::Array(view, conversed) => ViewPart::Views(view, conversed),
                    })
                }
                ViewPart::Element(mode) => Some(ViewPart::Element(mode)),
                ViewPart::Views(..) => None,
            },
            Name::CallOrIndexed(call) => match self.view_part(&call.name.item)? {
                ViewPart::Views(view, conversed) => Some(ViewPart::View(view, conversed)),
                part => Some(part),
            },
            Name::Slice(prefix, _) => self.view_part(&prefix.item),
            _ => None,
        }
    }
}

/// A part of a port with a mode view
enum ViewPart<'a> {
    /// A record element with a simple mode
    Element(Mode),
    /// A record with a view
    View(ViewEnt<'a>, bool),
    /// An array of records with a view
    Views(ViewEnt<'a>, bool),
}

impl ViewPart<'_> {
    fn can_be_assigned_to(&self) -> bool {
        match self {
            ViewPart::Element(mode) => *mode != Mode::In,
            ViewPart::View(view, conversed) | ViewPart::Views(view, conversed) => {
                !view.view().has_element_with_mode(Mode::In, *conversed)
            }
        }
    }

    fn describe_illegal_target(&self, target: &Name) -> String {
        match self {
            ViewPart::Element(_) => {
                format!("'{target}' has mode in and may not be the target of an assignment")
            }
            ViewPart::View(..) | ViewPart::Views(..) => format!(
                "'{target}' has elements of mode in and may not be the target of an assignment"
            ),
        }
    }
}

#[derive(Copy, Clone)]
pub enum AssignmentType {
    // Assignment with <=
    Signal,
    // Assignment with :=
    Variable,
}

impl AssignmentType {
    fn to_str(self) -> &'static str {
        match self {
            AssignmentType::Signal => "signal",
            AssignmentType::Variable => "variable",
        }
    }
}
