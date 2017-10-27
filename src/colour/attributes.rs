// Copyright 2017 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cairo::{self, Gradient};
use gtk;
use gtk::prelude::*;
use gtk::WidgetExt;

use colour::*;
use cairox::*;
use pwo::*;
use rgb_math::angle::*;

type ColourStops = Vec<[f64; 4]>;

pub trait ColourAttributeDisplayInterface: PackableWidgetInterface {
    type CADIType;

    fn create() -> Self::CADIType;

    fn set_colour(&self, colour: Option<&Colour>);
    fn attr_value(&self) -> Option<f64>;
    fn attr_value_fg_rgb(&self) -> RGB;

    fn set_target_colour(&self, target_colour: Option<&Colour>);
    fn attr_target_value(&self) -> Option<f64>;
    fn attr_target_value_fg_rgb(&self) -> RGB;

    fn label(&self) -> &str;

    fn label_colour(&self) -> RGB {
        match self.attr_value() {
            Some(_) => self.attr_value_fg_rgb(),
            None => match self.attr_target_value() {
                Some(_) => self.attr_target_value_fg_rgb(),
                None => BLACK,
            },
        }
    }

    fn draw_attr_value_indicator(
        &self,
        drawing_area: &gtk::DrawingArea,
        cairo_context: &cairo::Context,
    ) {
        if let Some(attr_value) = self.attr_value() {
            let width = drawing_area.get_allocated_width() as f64;
            let height = drawing_area.get_allocated_height() as f64;
            let indicator_x = width * attr_value;
            cairo_context.set_source_colour_rgb(&self.attr_value_fg_rgb());
            cairo_context.draw_indicator(Point(indicator_x, 1.0), Side::Top, 8.0);
            cairo_context.draw_indicator(Point(indicator_x, height - 1.0), Side::Bottom, 8.0);
        }
    }

    fn draw_attr_target_value_indicator(
        &self,
        drawing_area: &gtk::DrawingArea,
        cairo_context: &cairo::Context,
    ) {
        if let Some(attr_target_value) = self.attr_target_value() {
            let width = drawing_area.get_allocated_width() as f64;
            let height = drawing_area.get_allocated_height() as f64;
            let target_x = width * attr_target_value;
            cairo_context.set_line_width(2.0);
            cairo_context.set_source_colour_rgb(&self.attr_target_value_fg_rgb());
            cairo_context.draw_line(Point(target_x, 0.0), Point(target_x, height));
        }
    }

    fn draw_label(
        &self,
        drawing_area: &gtk::DrawingArea,
        cairo_context: &cairo::Context
    ) {
        let label = self.label();
        if label.len() > 0 {
            let width = drawing_area.get_allocated_width() as f64;
            let height = drawing_area.get_allocated_height() as f64;
            cairo_context.set_font_size(15.0);
            let te = cairo_context.text_extents(&label);
            let x = (width - te.width) / 2.0;
            let y = (height + te.height) / 2.0;
            cairo_context.move_to(x, y);
            cairo_context.set_source_colour_rgb(&self.label_colour());
            cairo_context.show_text(&label);
        }
    }

    fn colour_stops(&self) -> ColourStops {
        vec![[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]]
    }

    fn draw_background(
        &self,
        drawing_area: &gtk::DrawingArea,
        cairo_context: &cairo::Context
    ) {
        let width = drawing_area.get_allocated_width() as f64;
        let height = drawing_area.get_allocated_height() as f64;
        let linear_gradient = cairo::LinearGradient::new(0.0, 0.0, width, height);
        for colour_stop in self.colour_stops() {
            linear_gradient.add_color_stop_rgb(
                colour_stop[0],
                colour_stop[1],
                colour_stop[2],
                colour_stop[3],
            );
        }
        cairo_context.rectangle(0.0, 0.0, width, height);
        cairo_context.set_source(&linear_gradient);
        cairo_context.fill()
    }

    fn draw_all(
        &self,
        drawing_area: &gtk::DrawingArea,
        cairo_context: &cairo::Context
    ) {
        self.draw_background(drawing_area, cairo_context);
        self.draw_attr_target_value_indicator(drawing_area, cairo_context);
        self.draw_attr_value_indicator(drawing_area, cairo_context);
        self.draw_label(drawing_area, cairo_context);
    }
}

// VALUE
#[derive(Debug)]
pub struct ValueCADData {
    drawing_area: gtk::DrawingArea,
    attr_value: Cell<Option<f64>>,
    attr_target_value: Cell<Option<f64>>,
    attr_value_fg_rgb: Cell<RGB>,
    attr_target_value_fg_rgb: Cell<RGB>,
}

pub type ValueCAD = Rc<ValueCADData>;

implement_pwo!(ValueCAD, drawing_area, gtk::DrawingArea);

impl ColourAttributeDisplayInterface for ValueCAD {
    type CADIType = ValueCAD;

    fn create() -> ValueCAD {
        let value_cad = Rc::new(
            ValueCADData {
                drawing_area: gtk::DrawingArea::new(),
                attr_value: Cell::new(None),
                attr_target_value: Cell::new(None),
                attr_value_fg_rgb: Cell::new(BLACK),
                attr_target_value_fg_rgb: Cell::new(BLACK),
            }
        );
        let value_cad_c = value_cad.clone();
        value_cad.drawing_area.connect_draw(
            move |da, ctxt|
            {
                value_cad_c.draw_all(da, ctxt);
                Inhibit(false)
            }
        );
        value_cad
    }

    fn set_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            self.attr_value.set(Some(colour.value()));
            self.attr_value_fg_rgb
                .set(colour.monotone_rgb().best_foreground_rgb());
        } else {
            self.attr_value.set(None);
            self.attr_value_fg_rgb.set(BLACK);
        }
        self.drawing_area.queue_draw()
    }

    fn attr_value(&self) -> Option<f64> {
        self.attr_value.get()
    }

    fn attr_value_fg_rgb(&self) -> RGB {
        self.attr_value_fg_rgb.get()
    }

    fn set_target_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            self.attr_target_value.set(Some(colour.value()));
            self.attr_target_value_fg_rgb
                .set(colour.monotone_rgb().best_foreground_rgb());
        } else {
            self.attr_target_value.set(None);
            self.attr_target_value_fg_rgb.set(BLACK);
        }
        self.drawing_area.queue_draw()
    }

    fn attr_target_value(&self) -> Option<f64> {
        self.attr_target_value.get()
    }

    fn attr_target_value_fg_rgb(&self) -> RGB {
        self.attr_target_value_fg_rgb.get()
    }

    fn label(&self) -> &str {
        "Value"
    }
}

// HUE
#[derive(Debug)]
pub struct HueCADData {
    drawing_area: gtk::DrawingArea,
    value_angle: Cell<Option<Angle>>,
    target_angle: Cell<Option<Angle>>,
    attr_value: Cell<Option<f64>>,
    attr_value_fg_rgb: Cell<RGB>,
    attr_target_value_fg_rgb: Cell<RGB>,
    colour_stops: RefCell<ColourStops>,
}

impl HueCADData {
    fn set_colour_stops(&self, ocolour: Option<&Colour>) {
        *self.colour_stops.borrow_mut() = if let Some(ref colour) = ocolour {
            if colour.is_grey() {
                let value = colour.value();
                vec![[0.0, value, value, value], [1.0, value, value, value]]
            } else {
                let mut stops: ColourStops = Vec::new();
                let mut hue_angle = colour.hue() + DEG_180;
                let delta_angle = DEG_180 / 6;
                for i in 0..13 {
                    let offset = i as f64 / 12.0;
                    let rgb = hue_angle.max_chroma_rgb();
                    stops.push([offset, rgb[0], rgb[1], rgb[2]]);
                    hue_angle = hue_angle + delta_angle
                }
                stops
            }
        } else {
            vec![[0.0, 0.5, 0.5, 0.5], [1.0, 0.5, 0.5, 0.5]]
        }
    }

    fn set_hue_defaults(&self) {
        self.value_angle.set(None);
        self.attr_value.set(None);
        if self.target_angle.get().is_none() {
            self.set_colour_stops(None);
        }
    }

    fn set_target_defaults(&self) {
        self.target_angle.set(None);
        if self.value_angle.get().is_none() {
            self.set_colour_stops(None);
        }
    }
}

fn calc_hue_value(hue_angle: Angle, target_angle: Angle) -> f64 {
    0.5  + (target_angle - hue_angle) / DEG_360
}

pub type HueCAD = Rc<HueCADData>;

implement_pwo!(HueCAD, drawing_area, gtk::DrawingArea);

impl ColourAttributeDisplayInterface for HueCAD {
    type CADIType = HueCAD;

    fn create() -> HueCAD {
        let hue_cad = Rc::new(
            HueCADData {
                drawing_area: gtk::DrawingArea::new(),
                value_angle: Cell::new(None),
                target_angle: Cell::new(None),
                attr_value: Cell::new(None),
                attr_value_fg_rgb: Cell::new(BLACK),
                attr_target_value_fg_rgb: Cell::new(BLACK),
                colour_stops: RefCell::new(vec![[0.0, 0.5, 0.5, 0.5], [1.0, 0.5, 0.5, 0.5]]),
            }
        );
        let hue_cad_c = hue_cad.clone();
        hue_cad.drawing_area.connect_draw(
            move |da, ctxt|
            {
                hue_cad_c.draw_all(da, ctxt);
                Inhibit(false)
            }
        );
        hue_cad
    }

    fn set_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            if colour.is_grey() {
                self.set_hue_defaults();
            } else {
                let val_angle = colour.hue.angle();
                self.value_angle.set(Some(val_angle));
                self.attr_value_fg_rgb
                    .set(colour.best_foreground_rgb());
                if let Some(target_angle) = self.target_angle.get() {
                    let val = calc_hue_value(target_angle, val_angle);
                    self.attr_value.set(Some(val));
                } else {
                    self.set_colour_stops(Some(&colour));
                    self.attr_value.set(Some(0.5))
                }
            }
        } else {
            self.set_hue_defaults();
        }
        self.drawing_area.queue_draw()
    }

    fn attr_value(&self) -> Option<f64> {
        self.attr_value.get()
    }

    fn attr_value_fg_rgb(&self) -> RGB {
        self.attr_value_fg_rgb.get()
    }

    fn set_target_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            if colour.is_grey() {
                self.set_target_defaults();
            } else {
                let target_angle = colour.hue.angle();
                self.target_angle.set(Some(target_angle));
                self.attr_target_value_fg_rgb
                    .set(colour.best_foreground_rgb());
                if let Some(val_angle) = self.value_angle.get() {
                    let val = calc_hue_value(target_angle, val_angle);
                    self.attr_value.set(Some(val));
                }
                self.set_colour_stops(Some(&colour));
            }
        } else {
            self.set_target_defaults();
        }
        self.drawing_area.queue_draw()
    }

    fn attr_target_value(&self) -> Option<f64> {
        if self.target_angle.get().is_some() {
            Some(0.5)
        } else {
            None
        }
    }

    fn attr_target_value_fg_rgb(&self) -> RGB {
        self.attr_target_value_fg_rgb.get()
    }

    fn colour_stops(&self) -> ColourStops {
        self.colour_stops.borrow().clone()
    }

    fn label(&self) -> &str {
        "Hue"
    }
}

// CHROMA
#[derive(Debug)]
pub struct ChromaCADData {
    drawing_area: gtk::DrawingArea,
    attr_value: Cell<Option<f64>>,
    attr_value_fg_rgb: Cell<RGB>,
    attr_target_value: Cell<Option<f64>>,
    attr_target_value_fg_rgb: Cell<RGB>,
    colour_stops: RefCell<ColourStops>,
}

impl ChromaCADData {
    fn set_colour_stops(&self, ocolour: Option<&Colour>) {
        *self.colour_stops.borrow_mut() = if let Some(ref colour) = ocolour {
            if colour.is_grey() {
                let value = colour.value();
                vec![[0.0, value, value, value], [1.0, value, value, value]]
            } else {
                let start_rgb = WHITE * colour.value();
                let end_rgb = colour.max_chroma_rgb();
                vec![
                    [0.0, start_rgb[0], start_rgb[1], start_rgb[2]],
                    [1.0, end_rgb[0], end_rgb[1], end_rgb[2]],
                ]
            }
        } else {
            vec![[0.0, 0.5, 0.5, 0.5], [1.0, 0.5, 0.5, 0.5]]
        }
    }

    fn set_chroma_defaults(&self) {
        self.attr_value.set(None);
        if self.attr_target_value.get().is_none() {
            self.set_colour_stops(None)
        }
    }

    fn set_target_defaults(&self) {
        self.attr_target_value.set(None);
        if self.attr_value.get().is_none() {
            self.set_colour_stops(None)
        }
    }
}

pub type ChromaCAD = Rc<ChromaCADData>;

implement_pwo!(ChromaCAD, drawing_area, gtk::DrawingArea);

impl ColourAttributeDisplayInterface for ChromaCAD {
    type CADIType = ChromaCAD;

    fn create() -> ChromaCAD {
        let croma_cad = Rc::new(
            ChromaCADData {
                drawing_area: gtk::DrawingArea::new(),
                attr_value: Cell::new(None),
                attr_value_fg_rgb: Cell::new(BLACK),
                attr_target_value: Cell::new(None),
                attr_target_value_fg_rgb: Cell::new(BLACK),
                colour_stops: RefCell::new(vec![[0.0, 0.5, 0.5, 0.5], [1.0, 0.5, 0.5, 0.5]]),
            }
        );
        let croma_cad_c = croma_cad.clone();
        croma_cad.drawing_area.connect_draw(
            move |da, ctxt|
            {
                croma_cad_c.draw_all(da, ctxt);
                Inhibit(false)
            }
        );
        croma_cad
    }

    fn set_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            self.attr_value.set(Some(colour.chroma()));
            self.attr_value_fg_rgb
                .set(colour.best_foreground_rgb());
            if let Some(target_value) = self.attr_target_value.get() {
                if target_value == 0.0 {
                    self.set_colour_stops(Some(&colour));
                }
            } else {
                self.set_colour_stops(Some(&colour));
            }
        } else {
            self.set_chroma_defaults();
        }
        self.drawing_area.queue_draw()
    }

    fn attr_value(&self) -> Option<f64> {
        self.attr_value.get()
    }

    fn attr_value_fg_rgb(&self) -> RGB {
        self.attr_value_fg_rgb.get()
    }

    fn set_target_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            self.attr_target_value.set(Some(colour.chroma()));
            self.attr_target_value_fg_rgb
                .set(colour.best_foreground_rgb());
            if colour.is_grey() {
                if let Some(attr_value) = self.attr_value.get() {
                    if attr_value == 0.0 {
                        self.set_colour_stops(Some(&colour));
                    }
                } else {
                    self.set_colour_stops(Some(&colour));
                }
            } else {
                self.set_colour_stops(Some(&colour));
            }
        } else {
            self.set_target_defaults();
        }
        self.drawing_area.queue_draw()
    }

    fn attr_target_value(&self) -> Option<f64> {
        self.attr_target_value.get()
    }

    fn attr_target_value_fg_rgb(&self) -> RGB {
        self.attr_target_value_fg_rgb.get()
    }

    fn colour_stops(&self) -> ColourStops {
        self.colour_stops.borrow().clone()
    }

    fn label(&self) -> &str {
        "Chroma"
    }
}

pub trait ColourAttributeDisplayStackInterface: PackableWidgetInterface {
    fn create() -> Self;

    fn set_colour(&self, colour: Option<&Colour>);
    fn set_target_colour(&self, target_colour: Option<&Colour>);
}

pub struct HueChromaValueCADSData {
    vbox: gtk::Box,
    hue_cad: HueCAD,
    chroma_cad: ChromaCAD,
    value_cad: ValueCAD,
}

pub type HueChromaValueCADS = Rc<HueChromaValueCADSData>;

implement_pwo!(HueChromaValueCADS, vbox, gtk::Box);

impl ColourAttributeDisplayStackInterface for HueChromaValueCADS {
    fn create() -> HueChromaValueCADS {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let hue_cad = HueCAD::create();
        let chroma_cad = ChromaCAD::create();
        let value_cad = ValueCAD::create();
        vbox.pack_start(&hue_cad.pwo(), true, true, 0);
        vbox.pack_start(&chroma_cad.pwo(), true, true, 0);
        vbox.pack_start(&value_cad.pwo(), true, true, 0);
        Rc::new(
            HueChromaValueCADSData {
                vbox,
                hue_cad,
                chroma_cad,
                value_cad,
            }
        )
    }

    fn set_colour(&self, colour: Option<&Colour>) {
        self.hue_cad.set_colour(colour);
        self.chroma_cad.set_colour(colour);
        self.value_cad.set_colour(colour);
    }

    fn set_target_colour(&self, target_colour: Option<&Colour>) {
        self.hue_cad.set_target_colour(target_colour);
        self.chroma_cad.set_target_colour(target_colour);
        self.value_cad.set_target_colour(target_colour);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_attributes() {
        if !gtk::is_initialized() {
            if let Err(err) = gtk::init() {
                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
            };
        }

        let vcad = ValueCAD::create();

        vcad.set_colour(Some(&Colour::from(RED)));
        vcad.set_target_colour(Some(&Colour::from(BLUE)));

        let hcad = HueCAD::create();

        hcad.set_colour(Some(&Colour::from(RED)));
        hcad.set_target_colour(Some(&Colour::from(BLUE)));

        let ccad = ChromaCAD::create();

        ccad.set_colour(Some(&Colour::from(RED)));
        ccad.set_target_colour(Some(&Colour::from(BLUE)));

        let hcv_cads  = HueChromaValueCADS::create();
        hcv_cads.set_colour(Some(&Colour::from(RED)));
    }
}
