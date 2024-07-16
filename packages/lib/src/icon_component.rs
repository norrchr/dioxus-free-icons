use core::str;

use dioxus::prelude::*;

/// Icon shape trait
pub trait IconShape {
    fn view_box(&self) -> &str;
    fn width(&self) -> &str;
    fn height(&self) -> &str;
    fn xmlns(&self) -> &str;
    fn title(&self) -> &str;
    fn child_elements(&self) -> Element;
    fn fill(&self) -> &str;
    fn stroke(&self) -> &str;
    fn stroke_width(&self) -> &str;
    fn stroke_linecap(&self) -> &str;
    fn stroke_linejoin(&self) -> &str;
}

/// Icon component Props
#[derive(PartialEq, Props, Clone)]
pub struct IconProps<T: IconShape + Clone + PartialEq + 'static> {
    /// The icon shape to use.
    pub icon: T,
    /// The id of the `<svg>` element.
    #[props(default = None)]
    pub id: Option<String>,
    /// The height of the `<svg>` element. Defaults to the icon's default height.
    #[props(default = 0)]
    pub height: u32,
    /// The width of the `<svg>` element. Defaults to the icon's default width.
    #[props(default = 0)]
    pub width: u32,
    /// The color to use for filling the icon. Defaults to the icon's default fill color.
    #[props(default = None)]
    pub fill: Option<String>,
    /// The color to use for strokeing the icon. Defaults to the icon's default stroke color.
    #[props(default = None)]
    pub stroke: Option<String>,
    /// The width of the stroke. Defaults to the icon's default stroke width.
    #[props(default = None)]
    pub stroke_width: Option<f32>,
    /// The linecap style of the stroke. Defaults to the icon's default stroke linecap.
    #[props(default = None)]
    pub stroke_linecap: Option<String>,
    /// The linejoin style of the stroke. Defaults to the icon's default stroke linejoin.
    #[props(default = None)]
    pub stroke_linejoin: Option<String>,
    /// An class for the `<svg>` element.
    #[props(default = "".to_string())]
    pub class: String,
    /// An accessible, short-text description for the icon. Defaults to the icon's default title.
    /// If the icon's title is empty, no title element will be generated.
    #[props(default = None)]
    pub title: Option<String>,
}

/// Icon component which generates SVG elements
#[allow(non_snake_case)]
pub fn Icon<T: IconShape + Clone + PartialEq + 'static>(props: IconProps<T>) -> Element {

    let id = props.id.unwrap_or_default();
    let width = if props.width == 0 { props.icon.width() } else { &props.width.to_string() };
    let height = if props.height == 0 { props.icon.height() } else { &props.height.to_string() };
    let fill = props.fill.as_deref().unwrap_or(props.icon.fill());
    let stroke = props.stroke.as_deref().unwrap_or(props.icon.stroke());
    let stroke_width = props.stroke_width.map(|v| v.to_string()).unwrap_or(props.icon.stroke_width().to_string());
    let stroke_linecap = props.stroke_linecap.as_deref().unwrap_or(props.icon.stroke_linecap());
    let stroke_linejoin = props.stroke_linejoin.as_deref().unwrap_or(props.icon.stroke_linejoin());
    let title_text = props.title.as_deref().unwrap_or(props.icon.title());

    rsx!(
        svg {
            id,
            class: "{props.class}",
            height,
            width,
            view_box: "{props.icon.view_box()}",
            xmlns: "{props.icon.xmlns()}",
            "xmlns:xlink": "http://www.w3.org/1999/xlink",
            fill,
            stroke,
            stroke_width,
            stroke_linecap,
            stroke_linejoin,
            if !title_text.is_empty() {
                title {
                    "{title_text}"
                }
            },
            {props.icon.child_elements()},
        }
    )
}
