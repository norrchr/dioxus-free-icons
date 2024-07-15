use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use ego_tree::{NodeRef,iter::Children};
use heck::{ToSnakeCase, ToUpperCamelCase};
use regex::Regex;
use scraper::{Html, Node, node::Element};
use walkdir::WalkDir;

const ICON_TEMPLATE: &str = r#"#[derive(Copy, Clone, Debug, PartialEq)]
pub struct {ICON_NAME};
impl IconShape for {ICON_NAME} {
    fn view_box(&self) -> &str {
        "{VIEW_BOX}"
    }
    fn width(&self) -> &str {
        "{WIDTH}"
    }
    fn height(&self) -> &str {
        "{HEIGHT}"
    }
    fn xmlns(&self) -> &str {
        "{XMLNS}"
    }
    fn fill(&self) -> &str {
        "{FILL_COLOR}"
    }
    fn stroke(&self) -> &str {
        "{STROKE_COLOR}"
    }
    fn stroke_width(&self) -> &str {
        "{STROKE_WIDTH}"
    }
    fn stroke_linecap(&self) -> &str {
        "{STROKE_LINECAP}"
    }
    fn stroke_linejoin(&self) -> &str {
        "{STROKE_LINEJOIN}"
    }
    fn title(&self) -> &str {
        "{TITLE}"
    }
    fn child_elements(&self) -> Element {
        rsx! {
{CHILD_ELEMENTS}
        }
    }
}
"#;

pub fn create_icon_file(svg_path: &str, output_path: &str, icon_prefix: &str) {
    let files = collect_svg_files(svg_path, icon_prefix);
    let mut processed_icons = 0;

    let icon_file = files
        .into_iter()
        .map(|file| {
            let content = fs::read_to_string(&file).unwrap();
            let fragment = Html::parse_fragment(&content);
            
            // find the svg node in the fragment tree
            let svg_node: Option<NodeRef<Node>> = fragment.tree.nodes().find(|node| {
                if let Some(element) = node.value().as_element() {
                    return element.name() == "svg" && node.has_children()
                }
                false
            });

            if let Some(svg_node) = svg_node {

                // this is the svg element
                let svg_element = svg_node.value().as_element().unwrap();

                let icon_name = icon_name(&file, icon_prefix);
                let (view_box, xmlns) = extract_svg_attrs(svg_element);
                let (width, height) = extract_svg_dimensions(svg_element);
                let (fill_color, stroke_color, stroke_width) = extract_svg_colors(svg_element);
                let stroke_linecap = extract_svg_stroke_linecap(svg_element);
                let stroke_linejoin = extract_svg_stroke_linejoin(svg_element);

                let mut child_elements = String::new();
                let mut title = String::new();
                extract_svg_child_nodes(0, svg_node.children(), &mut title, &mut child_elements, icon_prefix);

                processed_icons += 1;

                ICON_TEMPLATE
                    .replace("{ICON_NAME}", &format!("{}{}", icon_prefix, &icon_name))
                    .replace("{VIEW_BOX}", &view_box)
                    .replace("{WIDTH}", &width)
                    .replace("{HEIGHT}", &height)
                    .replace("{XMLNS}", &xmlns)
                    .replace("{TITLE}", &title.trim())
                    .replace("{CHILD_ELEMENTS}", &child_elements.trim_end())
                    .replace("{FILL_COLOR}", &fill_color)
                    .replace("{STROKE_COLOR}", &stroke_color)
                    .replace("{STROKE_WIDTH}", &stroke_width)
                    .replace("{STROKE_LINECAP}", &stroke_linecap)
                    .replace("{STROKE_LINEJOIN}", &stroke_linejoin)

            } else {
                panic!("no svg node found in file: {:?}", file);
            }

        })
        .collect::<Vec<_>>()
        .join("\n");

    // write to file
    let mut file = File::create(output_path).unwrap();
    file.write_all(
        format!(
            "{}\n\n{}",
            "use super::super::IconShape;\nuse dioxus::prelude::*;", icon_file
        )
        .as_bytes(),
    )
    .unwrap();
    file.flush().unwrap();


    println!("[{icon_prefix}] Generated {processed_icons} icon(s) at: {output_path}");

}

fn collect_svg_files(svg_path: &str, icon_prefix: &str) -> Vec<PathBuf> {
    let dir_entries = WalkDir::new(svg_path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    dir_entries
        .into_iter()
        .filter(|e| match icon_prefix {
            "Go" => {
                let re = Regex::new(r".*-16.svg$").unwrap();
                return re.is_match(e.path().to_str().unwrap());
            }
            "Md" => {
                let split_vec = e.path().components().collect::<Vec<_>>();
                return split_vec.iter().any(|c| c.as_os_str() == "materialicons")
                    && e.file_name().to_str().unwrap() == "24px.svg";
            }
            _ => return e.path().extension() == Some(OsStr::new("svg")),
        })
        .map(|dir| PathBuf::from(dir.path()))
        .collect::<Vec<_>>()
}

fn icon_name(path: &Path, icon_prefix: &str) -> String {
    match icon_prefix {
        "Go" => {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let name = filename.split('.').next().unwrap();
            name.replace("-16", "").to_upper_camel_case()
        }
        "Md" => {
            let split_vec = path.components().collect::<Vec<_>>();
            let name = split_vec[split_vec.len() - 3];
            name.as_os_str().to_str().unwrap().to_upper_camel_case()
        }
        _ => {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let name = filename.split('.').next().unwrap();
            name.to_upper_camel_case()
        }
    }
}

fn extract_svg_attrs(element: &Element) -> (&str, &str) {
    (
        element.attr("viewBox").unwrap(),
        element.attr("xmlns").unwrap_or("http://www.w3.org/2000/svg")
    )
}

fn extract_svg_colors(element: &Element) -> (&str, &str, &str) {
    (
        element.attr("fill").unwrap_or("black"),
        element.attr("stroke").unwrap_or("none"),
        element.attr("stroke-width").unwrap_or("1"),
    )
}

fn extract_svg_dimensions(element: &Element) -> (&str, &str) {
    (
        element.attr("width").unwrap_or("300"),
        element.attr("height").unwrap_or("150"),
    )
}

fn extract_svg_stroke_linecap(element: &Element) -> &str {
    element.attr("stroke-linecap").unwrap_or("butt")
}

fn extract_svg_stroke_linejoin(element: &Element) -> &str {
    element.attr("stroke-linejoin").unwrap_or("miter")
}

fn convert_element_attributes(element: &Element, icon_prefix: &str) -> String {
    let ignore_attrs: Vec<&str> = vec![
        #[cfg(feature = "strip-id")] "id",
        #[cfg(feature = "strip-class")] "class",
        #[cfg(feature = "strip-fill")] "fill",
        #[cfg(feature = "strip-stroke")] "stroke",
    ];

    let mut element_attrs: Vec<String> = element
        .attrs()
        .filter_map(|(name, value)| {
            let value = if icon_prefix == "Io" {
                value.replace("fill:none;stroke:#000;", "")
            } else {
                value.to_string()
            };
            if value.is_empty() { return None };
            let re = Regex::new(r"^data-.*$").unwrap();

            if !re.is_match(name) {
                // force fill on group elements to 'currentColor'
                #[cfg(feature = "g-force-fill-currentcolor")]
                if element.name() == "g" && name == "fill" {
                    return Some(format!("{}: \"{}\",", name.to_snake_case(), "currentColor"))
                }
                
                // allow fill if value is currentColor
                #[cfg(feature = "allow-fill-currentcolor")]
                if name == "fill" && value.to_lowercase() == "currentcolor" {
                    return Some(format!("{}: \"{}\",", name.to_snake_case(), value))
                }

                if icon_prefix == "Md" && (element.name() == "rect" || element.name() == "path") && name == "fill" {
                    return Some(format!("{}: \"{}\",", name.to_snake_case(), value))
                }
                
                // allow if attribute is not in ignore_attrs
                if !ignore_attrs.contains(&name) {
                    return Some(format!("{}: \"{}\",", name.to_snake_case(), value))
                }
            }
            None
        })
        .collect::<Vec<_>>();

    element_attrs.sort();
    element_attrs.join("\n")
}

fn children_are_elements(children: Children<Node>) -> bool {
    children.filter(|node| node.value().is_element()).count() > 0
}


fn extract_svg_child_nodes(layer: usize, children: Children<Node>, title: &mut String, result: &mut String, icon_prefix: &str) {
    let indent = "    ".repeat(layer) + "            ";

    children.filter(|node| node.value().is_element()).for_each(|child_node| {
        let element = child_node.value().as_element().unwrap();
        let attrs = convert_element_attributes(element, icon_prefix);

        if element.name() == "title" {
            if let Some(title_node) = child_node.first_child() {
                if let Some(title_text) = title_node.value().as_text() {
                    if title.is_empty() {
                        title.push_str(&title_text.to_string())
                    }
                }
            }
            return;
        } else if element.name() == "g" {
            if !attrs.is_empty() && children_are_elements(child_node.children()) {
                result.push_str(&format!("{indent}{} {{\n", element.name()));
                for attr in attrs.split('\n') {
                    result.push_str(&format!("{indent}    {}\n", attr));
                }
                extract_svg_child_nodes(layer + 1, child_node.children(), title, result, icon_prefix);
                result.push_str(&format!("{indent}}}\n"));
            } else {
                extract_svg_child_nodes(layer, child_node.children(), title, result, icon_prefix);
            }
        } else {
            result.push_str(&format!("{indent}{} {{\n", element.name()));
            for attr in attrs.split('\n') {
                result.push_str(&format!("{indent}    {}\n", attr));
            }
            if !element.attrs.is_empty() && child_node.has_children() {
                result.push_str("\n");
            }
            if child_node.has_children() {
                extract_svg_child_nodes(layer + 1, child_node.children(), title,result, icon_prefix);
            }
            result.push_str(&format!("{indent}}}\n"));
        }
    })

}
