/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2020 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2020 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */

use cpp::*;
use sixtyfps_corelib::component::{ComponentRc, ComponentWeak};
use sixtyfps_corelib::graphics::{GraphicsBackend, Point};
use sixtyfps_corelib::item_rendering::ItemRenderer;
use sixtyfps_corelib::items::{self, ItemRef};
use sixtyfps_corelib::properties::PropertyTracker;
use sixtyfps_corelib::window::GenericWindow;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

use super::qttypes;

cpp! {{
    #include <QtWidgets/QWidget>
    #include <QtGui/QPainter>
    #include <QtGui/QPaintEngine>
    #include <QtGui/QWindow>
    #include <QtGui/QResizeEvent>
    void ensure_initialized();

    struct SixtyFPSWidget : QWidget {
        void *rust_window;
        void paintEvent(QPaintEvent *) override {
            QPainter painter(this);
            auto painter_ptr = &painter;
            rust!(SFPS_paintEvent [rust_window: &QtWindow as "void*", painter_ptr: &mut QPainter as "QPainter*"] {
                rust_window.paint_event(painter_ptr)
            });
        }

        void resizeEvent(QResizeEvent *event) override {
            QSize size = event->size();
            rust!(SFPS_resizeEvent [rust_window: &QtWindow as "void*", size: qttypes::QSize as "QSize"] {
                rust_window.resize_event(size)
            });
        }

    };
}}

cpp_class! {pub unsafe struct QPainter as "QPainter"}

macro_rules! get_geometry {
    ($ty:ty, $obj:expr) => {{
        type Ty = $ty;
        let width = Ty::FIELD_OFFSETS.width.apply_pin($obj).get();
        let height = Ty::FIELD_OFFSETS.height.apply_pin($obj).get();
        let x = Ty::FIELD_OFFSETS.x.apply_pin($obj).get();
        let y = Ty::FIELD_OFFSETS.y.apply_pin($obj).get();
        if width < 1. || height < 1. {
            return Default::default();
        };
        qttypes::QRectF { x: x as _, y: y as _, width: width as _, height: height as _ }
    }};
}

macro_rules! get_pos {
    ($ty:ty, $obj:expr) => {{
        type Ty = $ty;
        let x = Ty::FIELD_OFFSETS.x.apply_pin($obj).get();
        let y = Ty::FIELD_OFFSETS.y.apply_pin($obj).get();
        qttypes::QPoint { x: x as _, y: y as _ }
    }};
}

impl ItemRenderer for QPainter {
    fn draw_rectangle(&mut self, pos: Point, rect: Pin<&items::Rectangle>) {
        let pos = qttypes::QPoint { x: pos.x as _, y: pos.y as _ };
        let color: u32 = rect.color().as_argb_encoded();
        let rect: qttypes::QRectF = get_geometry!(items::Rectangle, rect);
        cpp! { unsafe [self as "QPainter*", pos as "QPoint", color as "QRgb", rect as "QRectF"] {
            self->fillRect(rect.translated(pos), QColor::fromRgba(color));
        }}
    }

    fn draw_border_rectangle(&mut self, pos: Point, rect: std::pin::Pin<&items::BorderRectangle>) {
        let pos = qttypes::QPoint { x: pos.x as _, y: pos.y as _ };
        let color: u32 = rect.color().as_argb_encoded();
        let border_color: u32 = rect.border_color().as_argb_encoded();
        let border_width: f32 = rect.border_width();
        let radius: f32 = rect.border_radius();
        let rect: qttypes::QRectF = get_geometry!(items::BorderRectangle, rect);
        cpp! { unsafe [self as "QPainter*", pos as "QPoint", color as "QRgb",  border_color as "QRgb", border_width as "float", radius as "float", rect as "QRectF"] {
            self->setPen(border_width > 0 ? QPen(QColor::fromRgba(border_color), border_width) : Qt::NoPen);
            self->setBrush(QColor::fromRgba(color));
            if (radius > 0) {
                self->drawRoundedRect(rect.translated(pos), radius, radius);
            } else {
                self->drawRect(rect.translated(pos));
            }
        }}
    }

    fn draw_image(&mut self, pos: Point, image: std::pin::Pin<&items::Image>) {
        todo!()
    }

    fn draw_clipped_image(&mut self, pos: Point, image: std::pin::Pin<&items::ClippedImage>) {
        todo!()
    }

    fn draw_text(&mut self, pos: Point, text: std::pin::Pin<&items::Text>) {
        let pos1 = qttypes::QPoint { x: pos.x as _, y: pos.y as _ };
        let pos2: qttypes::QPoint = get_pos!(items::Text, text);
        let color: u32 = items::Text::FIELD_OFFSETS.color.apply_pin(text).get().as_argb_encoded();
        let string: qttypes::QString =
            items::Text::FIELD_OFFSETS.text.apply_pin(text).get().as_str().into();
        cpp! { unsafe [self as "QPainter*", pos1 as "QPoint", pos2 as "QPoint", color as "QRgb", string as "QString"] {
            self->setPen(QColor{color});
            self->setBrush(Qt::NoBrush);
            self->drawText(pos1 + pos2, string);
        }}
    }

    fn draw_text_input(&mut self, pos: Point, text_input: std::pin::Pin<&items::TextInput>) {
        let pos1 = qttypes::QPoint { x: pos.x as _, y: pos.y as _ };
        let pos2: qttypes::QPoint = get_pos!(items::TextInput, text_input);
        let color: u32 =
            items::TextInput::FIELD_OFFSETS.color.apply_pin(text_input).get().as_argb_encoded();
        let string: qttypes::QString =
            items::TextInput::FIELD_OFFSETS.text.apply_pin(text_input).get().as_str().into();
        cpp! { unsafe [self as "QPainter*", pos1 as "QPoint", pos2 as "QPoint", color as "QRgb", string as "QString"] {
            self->setPen(QColor{color});
            self->setBrush(Qt::NoBrush);
            self->drawText(pos1 + pos2, string);
        }}
    }

    fn draw_path(&mut self, pos: Point, path: std::pin::Pin<&items::Path>) {
        todo!()
    }

    fn combine_clip(&mut self, pos: Point, clip: &std::pin::Pin<&items::Clip>) {
        todo!()
    }

    fn clip_rects(&self) -> sixtyfps_corelib::SharedVector<sixtyfps_corelib::graphics::Rect> {
        // FIXME
        return Default::default();
    }

    fn reset_clip(
        &mut self,
        rects: sixtyfps_corelib::SharedVector<sixtyfps_corelib::graphics::Rect>,
    ) {
        let mut iter = rects.iter();
        if let Some(r) =
            iter.next().and_then(|first| iter.try_fold(*first, |acc, r| acc.intersection(r)))
        {
            let rect = qttypes::QRectF {
                x: r.origin.x as _,
                y: r.origin.y as _,
                width: r.size.width as _,
                height: r.size.height as _,
            };
            cpp! { unsafe [self as "QPainter*", rect as "QRectF"] {
                self->setClipRect(rect, Qt::ReplaceClip);
            }}
        } else {
            cpp! { unsafe [self as "QPainter*"] {
                self->setClipRect(QRect(), Qt::NoClip);
            }}
        }
    }

    fn scale_factor(&self) -> f32 {
        cpp! { unsafe [self as "QPainter*"] -> f32 as "float" {
            return self->paintEngine()->paintDevice()->devicePixelRatioF();
        }}
    }

    fn draw_cached_pixmap(
        &mut self,
        item_cache: &sixtyfps_corelib::item_rendering::CachedRenderingData,
        pos: Point,
        update_fn: &dyn Fn(&mut dyn FnMut(u32, u32, &[u8])),
    ) {
        // FIXME! draw_cached_pixmap is the wrong abstraction now
        update_fn(&mut |width: u32, height: u32, data: &[u8]| {
            let pos = qttypes::QPoint { x: pos.x as _, y: pos.y as _ };
            let data = data.as_ptr();
            cpp! { unsafe [self as "QPainter*", pos as "QPoint", width as "int", height as "int", data as "const unsigned char *"] {
                QImage img(data, width, height, width * 4, QImage::Format_ARGB32_Premultiplied);
                self->drawImage(pos, img);
            }}
        })
    }
}

cpp_class!(unsafe struct QWidgetPtr as "std::unique_ptr<QWidget>");

pub struct QtWindow {
    widget_ptr: QWidgetPtr,
    component: std::cell::RefCell<ComponentWeak>,
    /// Gets dirty when the layout restrictions, or some other property of the windows change
    meta_property_listener: Pin<Rc<PropertyTracker>>,
}

impl QtWindow {
    pub fn new() -> Rc<Self> {
        let widget_ptr = cpp! {unsafe [] -> QWidgetPtr as "std::unique_ptr<QWidget>" {
            ensure_initialized();
            return std::make_unique<SixtyFPSWidget>();
        }};
        let rc = Rc::new(QtWindow {
            widget_ptr,
            component: Default::default(),
            meta_property_listener: Rc::pin(Default::default()),
        });
        let widget_ptr = rc.widget_ptr();
        let rust_window = Rc::as_ptr(&rc);
        cpp! {unsafe [widget_ptr as "SixtyFPSWidget*", rust_window as "void*"]  {
            widget_ptr->rust_window = rust_window;
        }};
        rc
    }

    /// Return the QWidget*
    fn widget_ptr(&self) -> NonNull<()> {
        unsafe { std::mem::transmute_copy::<QWidgetPtr, NonNull<_>>(&self.widget_ptr) }
    }

    /// ### Candidate to be moved in corelib as this kind of duplicate GraphicsWindow::draw
    fn paint_event(&self, painter: &mut QPainter) {
        sixtyfps_corelib::animations::update_animations();

        let component_rc = self.component.borrow().upgrade().unwrap();
        let component = ComponentRc::borrow_pin(&component_rc);

        if self.meta_property_listener.as_ref().is_dirty() {
            self.meta_property_listener.as_ref().evaluate(|| {
                self.apply_geometry_constraint(component.as_ref().layout_info());
                component.as_ref().apply_layout(Default::default());

                let root_item = component.as_ref().get_item_ref(0);
                if let Some(window_item) = ItemRef::downcast_pin(root_item) {
                    self.apply_window_properties(window_item);
                }
            })
        }

        sixtyfps_corelib::item_rendering::render_component_items::<QtBackend>(
            &component_rc,
            painter,
            Point::default(),
        );
    }

    fn resize_event(&self, size: qttypes::QSize) {
        let component = self.component.borrow().upgrade().unwrap();
        let component = ComponentRc::borrow_pin(&component);
        let root_item = component.as_ref().get_item_ref(0);
        if let Some(window_item) = ItemRef::downcast_pin::<items::Window>(root_item) {
            window_item.width.set(size.width as _);
            window_item.height.set(size.height as _);
        }
    }

    /// Set the min/max sizes on the QWidget
    fn apply_geometry_constraint(&self, constraints: sixtyfps_corelib::layout::LayoutInfo) {
        let widget_ptr = self.widget_ptr();
        let min_width: f32 = constraints.min_width.min(constraints.max_width);
        let min_height: f32 = constraints.min_height.min(constraints.max_height);
        let mut max_width: f32 = constraints.max_width.max(constraints.min_width);
        let mut max_height: f32 = constraints.max_height.max(constraints.min_height);
        cpp! {unsafe [widget_ptr as "QWidget*",  min_width as "float", min_height as "float", mut max_width as "float", mut max_height as "float"] {
            widget_ptr->setMinimumSize(QSize(min_width, min_height));
            if (max_width > QWIDGETSIZE_MAX)
                max_width = QWIDGETSIZE_MAX;
            if (max_height > QWIDGETSIZE_MAX)
                max_height = QWIDGETSIZE_MAX;
            widget_ptr->setMaximumSize(QSize(max_width, max_height));
        }};
    }

    /// Apply windows property such as title to the QWidget*
    fn apply_window_properties(&self, window_item: Pin<&items::Window>) {
        let widget_ptr = self.widget_ptr();
        let title: qttypes::QString =
            items::Window::FIELD_OFFSETS.title.apply_pin(window_item).get().as_str().into();
        cpp! {unsafe [widget_ptr as "QWidget*",  title as "QString"] {
            widget_ptr->setWindowTitle(title);
        }};
    }
}

#[allow(unused)]
impl GenericWindow for QtWindow {
    fn set_component(self: Rc<Self>, component: &sixtyfps_corelib::component::ComponentRc) {
        *self.component.borrow_mut() = vtable::VRc::downgrade(&component)
    }

    fn draw(self: Rc<Self>) {
        todo!()
    }

    fn process_mouse_input(
        self: Rc<Self>,
        pos: Point,
        what: sixtyfps_corelib::input::MouseEventType,
    ) {
        todo!()
    }

    fn process_key_input(self: Rc<Self>, event: &sixtyfps_corelib::input::KeyEvent) {
        todo!()
    }

    fn run(self: Rc<Self>) {
        let widget_ptr = self.widget_ptr();
        cpp! {unsafe [widget_ptr as "QWidget*"] {
            widget_ptr->show();
            qApp->exec();
        }};
    }

    fn request_redraw(&self) {
        todo!()
    }

    fn scale_factor(&self) -> f32 {
        let widget_ptr = self.widget_ptr();
        cpp! {unsafe [widget_ptr as "QWidget*"] -> f32 as "float" {
            return widget_ptr->windowHandle()->devicePixelRatio();
        }}
    }

    fn set_scale_factor(&self, factor: f32) {
        todo!()
    }

    fn refresh_window_scale_factor(&self) {
        todo!()
    }

    fn set_width(&self, width: f32) {
        todo!()
    }

    fn set_height(&self, height: f32) {
        todo!()
    }

    fn get_geometry(&self) -> sixtyfps_corelib::graphics::Rect {
        todo!()
    }

    fn free_graphics_resources<'a>(
        self: Rc<Self>,
        items: &sixtyfps_corelib::slice::Slice<'a, std::pin::Pin<items::ItemRef<'a>>>,
    ) {
    }

    fn set_cursor_blink_binding(&self, prop: &sixtyfps_corelib::Property<bool>) {
        todo!()
    }

    fn current_keyboard_modifiers(&self) -> sixtyfps_corelib::input::KeyboardModifiers {
        todo!()
    }

    fn set_current_keyboard_modifiers(
        &self,
        modifiers: sixtyfps_corelib::input::KeyboardModifiers,
    ) {
        todo!()
    }

    fn set_focus_item(self: Rc<Self>, focus_item: &items::ItemRc) {
        todo!()
    }

    fn set_focus(self: Rc<Self>, have_focus: bool) {
        todo!()
    }

    fn show_popup(&self, popup: &sixtyfps_corelib::component::ComponentRc, position: Point) {
        todo!()
    }

    fn close_popup(&self) {
        todo!()
    }

    fn font(
        &self,
        request: sixtyfps_corelib::graphics::FontRequest,
    ) -> Option<Rc<dyn sixtyfps_corelib::graphics::Font>> {
        // FIXME
        None
    }
}

struct QtBackend;
impl GraphicsBackend for QtBackend {
    type ItemRenderer = QPainter;

    fn new_renderer(&mut self, clear_color: &sixtyfps_corelib::Color) -> Self::ItemRenderer {
        todo!()
    }

    fn flush_renderer(&mut self, renderer: Self::ItemRenderer) {
        todo!()
    }

    fn release_item_graphics_cache(
        &self,
        data: &sixtyfps_corelib::item_rendering::CachedRenderingData,
    ) {
        todo!()
    }

    fn font(
        &mut self,
        request: sixtyfps_corelib::graphics::FontRequest,
    ) -> Rc<dyn sixtyfps_corelib::graphics::Font> {
        todo!()
    }

    fn window(&self) -> &winit::window::Window {
        todo!()
    }
}
