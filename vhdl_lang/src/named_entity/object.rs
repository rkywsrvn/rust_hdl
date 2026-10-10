//! This Source Code Form is subject to the terms of the Mozilla Public
//! License, v. 2.0. If a copy of the MPL was not distributed with this file,
//! You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! Copyright (c) 2023, Olof Kraigher olof.kraigher@gmail.com

use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use super::*;
use crate::ast::InterfaceType;
use crate::ast::Mode;
use crate::ast::ObjectClass;

// A named entity that is known to be an object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectEnt<'a> {
    pub ent: EntRef<'a>,
}

impl<'a> ObjectEnt<'a> {
    pub fn from_any(ent: EntRef<'a>) -> Option<Self> {
        if matches!(ent.actual_kind(), AnyEntKind::Object(..)) {
            Some(Self { ent })
        } else {
            None
        }
    }

    pub fn kind(&self) -> &'a Object<'a> {
        if let AnyEntKind::Object(object) = self.ent.actual_kind() {
            object
        } else {
            unreachable!("ObjectEnt type invariant broken")
        }
    }

    pub fn inner(&self) -> EntRef<'a> {
        self.ent
    }

    pub fn type_mark(&self) -> TypeEnt<'a> {
        self.kind().subtype.type_mark()
    }

    pub fn class(&self) -> ObjectClass {
        self.object().class
    }

    pub fn mode(&self) -> Option<&InterfaceMode<'a>> {
        self.object().mode()
    }

    pub fn describe(&self) -> String {
        if let Some(ref iface) = self.object().iface {
            match iface {
                ObjectInterface::Generic => format!("generic '{}'", self.designator()),
                ObjectInterface::Parameter(mode) => {
                    if self.class() == ObjectClass::Constant {
                        format!("parameter '{}'", self.designator())
                    } else {
                        format!("{} '{}' : {}", self.class(), self.designator(), mode)
                    }
                }
                ObjectInterface::Port(mode) => format!("port '{}' : {}", self.designator(), mode),
            }
        } else {
            self.describe_name()
        }
    }

    pub fn describe_name(&self) -> String {
        format!("{} '{}'", self.class(), self.designator())
    }

    pub fn object(&self) -> &'a Object<'a> {
        if let AnyEntKind::Object(object) = self.ent.actual_kind() {
            object
        } else {
            unreachable!("Must be object");
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum InterfaceMode<'a> {
    Simple(Mode),
    View(ViewEnt<'a>),
}

impl Display for InterfaceMode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceMode::Simple(mode) => write!(f, "{mode}"),
            InterfaceMode::View(view) => write!(f, "view {}", view.ent.designator),
        }
    }
}

#[derive(Clone)]
pub enum ObjectInterface<'a> {
    Generic,
    Port(InterfaceMode<'a>),
    Parameter(InterfaceMode<'a>),
}

impl ObjectInterface<'_> {
    pub fn simple(typ: InterfaceType, mode: Mode) -> Self {
        match typ {
            // @TODO error on non-input mode
            InterfaceType::Generic => ObjectInterface::Generic,
            InterfaceType::Parameter => ObjectInterface::Parameter(InterfaceMode::Simple(mode)),
            InterfaceType::Port => ObjectInterface::Port(InterfaceMode::Simple(mode)),
        }
    }

    pub fn mode(&self) -> &InterfaceMode<'_> {
        match self {
            ObjectInterface::Generic => &InterfaceMode::Simple(Mode::In),
            ObjectInterface::Parameter(m) | ObjectInterface::Port(m) => m,
        }
    }

    pub fn typ(&self) -> InterfaceType {
        match self {
            ObjectInterface::Generic => InterfaceType::Generic,
            ObjectInterface::Parameter(..) => InterfaceType::Parameter,
            ObjectInterface::Port(..) => InterfaceType::Port,
        }
    }
}

/// An object or an interface object,
/// example signal, variable, constant
/// Is either an object (mode = None) or an interface object (mode = Some)
#[derive(Clone)]
pub struct Object<'a> {
    pub class: ObjectClass,
    pub iface: Option<ObjectInterface<'a>>,
    pub subtype: Subtype<'a>,
    pub has_default: bool,
}

impl<'a> Object<'a> {
    pub(crate) fn const_param(subtype: Subtype<'a>) -> Object<'a> {
        Object {
            class: ObjectClass::Constant,
            iface: Some(ObjectInterface::Parameter(InterfaceMode::Simple(Mode::In))),
            subtype,
            has_default: false,
        }
    }

    pub(crate) fn with_default(mut self) -> Self {
        self.has_default = true;
        self
    }

    pub fn is_signal(&self) -> bool {
        self.class == ObjectClass::Signal
    }

    pub fn is_constant(&self) -> bool {
        self.class == ObjectClass::Constant
    }

    pub fn is_port(&self) -> bool {
        matches!(self.iface, Some(ObjectInterface::Port(..)))
    }

    pub fn is_generic(&self) -> bool {
        matches!(self.iface, Some(ObjectInterface::Generic))
    }

    pub fn is_param(&self) -> bool {
        matches!(self.iface, Some(ObjectInterface::Parameter(_)))
    }

    pub fn mode(&self) -> Option<&InterfaceMode<'_>> {
        self.iface.as_ref().map(|i| i.mode())
    }
}

impl ObjectClass {
    pub fn describe(&self) -> &str {
        use ObjectClass::*;
        match self {
            Constant => "constant",
            Variable => "variable",
            Signal => "signal",
            SharedVariable => "shared variable",
        }
    }
}

impl<'a> Deref for ObjectEnt<'a> {
    type Target = EntRef<'a>;
    fn deref(&self) -> &Self::Target {
        &self.ent
    }
}

// A named entity that is known to be a view
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewEnt<'a> {
    pub ent: EntRef<'a>,
}

impl<'a> ViewEnt<'a> {
    pub fn from_any(ent: EntRef<'a>) -> Option<Self> {
        if matches!(ent.actual_kind(), AnyEntKind::View(..)) {
            Some(Self { ent })
        } else {
            None
        }
    }

    pub fn view(&self) -> &'a View<'a> {
        if let AnyEntKind::View(view) = self.ent.actual_kind() {
            view
        } else {
            unreachable!("ViewEnt type invariant broken")
        }
    }

    pub fn subtype(&self) -> &'a Subtype<'a> {
        &self.view().subtype
    }
}

/// LRM 6.5.2 Mode view declaration
#[derive(Clone)]
pub struct View<'a> {
    pub subtype: Subtype<'a>,
    pub elements: Vec<(Designator, ViewElementMode<'a>)>,
}

impl<'a> View<'a> {
    pub fn element_mode(&self, designator: &Designator) -> Option<ViewElementMode<'a>> {
        self.elements
            .iter()
            .find(|(element, _)| element == designator)
            .map(|(_, mode)| *mode)
    }

    /// The view that `'converse` denotes.
    pub fn converse(&self) -> View<'a> {
        View {
            subtype: self.subtype,
            elements: self
                .elements
                .iter()
                .map(|(designator, mode)| (designator.clone(), mode.converse()))
                .collect(),
        }
    }

    /// Returns true if any element, including elements of nested views, has the given mode.
    pub fn has_element_with_mode(&self, mode: Mode, conversed: bool) -> bool {
        self.elements
            .iter()
            .any(|(_, element)| match element.converse_if(conversed) {
                ViewElementMode::Simple(element_mode) => element_mode == mode,
                ViewElementMode::Record(view, nested_conversed)
                | ViewElementMode::Array(view, nested_conversed) => {
                    view.view().has_element_with_mode(mode, nested_conversed)
                }
            })
    }
}

/// The mode of a record element in a mode view.
/// Nested views carry a flag that is true if the view is conversed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewElementMode<'a> {
    Simple(Mode),
    Record(ViewEnt<'a>, bool),
    Array(ViewEnt<'a>, bool),
}

impl ViewElementMode<'_> {
    pub fn converse(self) -> Self {
        match self {
            ViewElementMode::Simple(mode) => ViewElementMode::Simple(mode.converse()),
            ViewElementMode::Record(view, conversed) => ViewElementMode::Record(view, !conversed),
            ViewElementMode::Array(view, conversed) => ViewElementMode::Array(view, !conversed),
        }
    }

    pub fn converse_if(self, conversed: bool) -> Self {
        if conversed {
            self.converse()
        } else {
            self
        }
    }
}

impl Mode {
    /// LRM 6.5.2: The converse of a mode.
    pub fn converse(self) -> Mode {
        match self {
            Mode::In => Mode::Out,
            Mode::Out | Mode::Buffer => Mode::In,
            Mode::InOut => Mode::InOut,
            Mode::Linkage => Mode::Linkage,
        }
    }
}
