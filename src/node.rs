use egui::{Color32, Ui};
use egui_snarl::{
    Snarl,
    ui::{PinInfo, SnarlViewer},
};
use serde::{Deserialize, Serialize};

use crate::transform::{Encoding, Transformer};

#[derive(Clone, Deserialize, Serialize)]
pub enum Node {
    Input {
        text: String,
    },
    Transform {
        transformer: Transformer,
        data: Data,
    },
}

impl Node {
    pub fn data(&self) -> Data {
        match self {
            Node::Input { text } => Data::Text(text.into()),
            Node::Transform { data, .. } => data.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum Data {
    Text(String),
    List(Vec<Data>),
    Error(String),
}

impl Data {
    pub fn max_str_len(&self) -> usize {
        match self {
            Data::Text(text) => text.len(),
            Data::List(data_vec) => data_vec.iter().map(|d| d.max_str_len()).max().unwrap_or(0),
            Data::Error(text) => text.len(),
        }
    }
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Text(text) => write!(f, "{text}"),
            Data::List(data_vec) => write!(f, "{data_vec:?}"),
            Data::Error(text) => write!(f, "{text}"),
        }
    }
}

pub struct NodeViewer;

impl SnarlViewer<Node> for NodeViewer {
    fn title(&mut self, node: &Node) -> String {
        match node {
            Node::Input { .. } => "Input",
            Node::Transform { transformer, .. } => match transformer {
                Transformer::Split { .. } => "Split",
                Transformer::Join { .. } => "Join",
                Transformer::Find { .. } => "Find",
                Transformer::Replace { .. } => "Replace",
                Transformer::Slice { .. } => "Slice",
                Transformer::Encode { encoding } => match encoding {
                    Encoding::Base64 => "Base64 Encode",
                    Encoding::Base64UrlSafe => "Base64 URL Safe Encode",
                    Encoding::URL => "URL Encode",
                },
                Transformer::Decode { encoding } => match encoding {
                    Encoding::Base64 => "Base64 Decode",
                    Encoding::Base64UrlSafe => "Base64 URL Safe Decode",
                    Encoding::URL => "URL Decode",
                },
                Transformer::Uppercase => "Uppercase",
                Transformer::Lowercase => "Lowercase",
            },
        }
        .into()
    }

    fn inputs(&mut self, node: &Node) -> usize {
        match node {
            Node::Input { .. } => 0,
            Node::Transform { .. } => 1,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) -> PinInfo {
        match &*pin.remotes {
            [] => match snarl[pin.id.node] {
                Node::Input { .. } => unreachable!("Input takes no inputs"),
                Node::Transform { .. } => {
                    ui.label("NO INPUT");
                    PinInfo::circle()
                }
            },
            [remote] => {
                let input_data = snarl[remote.node].data();
                match &mut snarl[pin.id.node] {
                    Node::Input { .. } => unreachable!("Out takes no inputs"),
                    Node::Transform { data, transformer } => {
                        *data = transformer.transform(&input_data);
                        ui.label(format!("{data:?}"));
                        match transformer {
                            Transformer::Slice { from, to } => {
                                ui.add(
                                    egui::DragValue::new(from).range(0..=input_data.max_str_len()),
                                );
                                ui.add(
                                    egui::DragValue::new(to)
                                        .range(*from..=input_data.max_str_len()),
                                );
                            }
                            _ => (),
                        }
                        color_pin(&input_data)
                    }
                }
            }
            _ => unreachable!("Too many inputs"),
        }
    }

    fn outputs(&mut self, node: &Node) -> usize {
        match node {
            Node::Input { .. } => 1,
            Node::Transform { .. } => 1,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) -> PinInfo {
        match &mut snarl[pin.id.node] {
            Node::Input { text } => {
                ui.add(egui::TextEdit::multiline(text));
                PinInfo::circle().with_fill(Color32::from_rgb(16, 255, 16))
            }
            Node::Transform { data, transformer } => {
                match transformer {
                    Transformer::Split { pattern } => {
                        ui.add(egui::TextEdit::singleline(pattern).hint_text("pattern"));
                    }
                    Transformer::Join { separator } => {
                        ui.add(egui::TextEdit::singleline(separator).hint_text("separator"));
                    }
                    Transformer::Find { pattern } => {
                        ui.add(egui::TextEdit::singleline(pattern).hint_text("pattern"));
                    }
                    Transformer::Replace { pattern, replacer } => {
                        ui.add(egui::TextEdit::singleline(replacer).hint_text("replacer"));
                        ui.add(egui::TextEdit::singleline(pattern).hint_text("pattern"));
                    }
                    Transformer::Encode { encoding } | Transformer::Decode { encoding } => {
                        ui.selectable_value(encoding, Encoding::Base64, "Base64");
                        ui.selectable_value(encoding, Encoding::Base64UrlSafe, "Base64 URL Safe");
                        ui.selectable_value(encoding, Encoding::URL, "URL");
                    }
                    _ => (),
                }
                color_pin(data)
            }
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<Node>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        ui.label("Add node");
        if ui.button("Input").clicked() {
            snarl.insert_node(
                pos,
                Node::Input {
                    text: String::new(),
                },
            );
            ui.close_menu();
        }
        if ui.button("Split").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::List(Vec::new()),
                    transformer: Transformer::Split {
                        pattern: String::new(),
                    },
                },
            );
            ui.close_menu();
        }
        if ui.button("Join").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::List(Vec::new()),
                    transformer: Transformer::Join {
                        separator: String::new(),
                    },
                },
            );
            ui.close_menu();
        }
        if ui.button("Find").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::List(Vec::new()),
                    transformer: Transformer::Find {
                        pattern: String::new(),
                    },
                },
            );
            ui.close_menu();
        }
        if ui.button("Replace").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::List(Vec::new()),
                    transformer: Transformer::Replace {
                        pattern: String::new(),
                        replacer: String::new(),
                    },
                },
            );
            ui.close_menu();
        }
        if ui.button("Slice").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::Text(String::new()),
                    transformer: Transformer::Slice { from: 0, to: 0 },
                },
            );
            ui.close_menu();
        }
        if ui.button("Encode").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::Text(String::new()),
                    transformer: Transformer::Encode {
                        encoding: Encoding::Base64,
                    },
                },
            );
            ui.close_menu();
        }
        if ui.button("Decode").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::Text(String::new()),
                    transformer: Transformer::Decode {
                        encoding: Encoding::Base64,
                    },
                },
            );
            ui.close_menu();
        }
        if ui.button("Uppercase").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::Text(String::new()),
                    transformer: Transformer::Uppercase,
                },
            );
            ui.close_menu();
        }
        if ui.button("Lowercase").clicked() {
            snarl.insert_node(
                pos,
                Node::Transform {
                    data: Data::Text(String::new()),
                    transformer: Transformer::Lowercase,
                },
            );
            ui.close_menu();
        }
    }

    fn has_node_menu(&mut self, _node: &Node) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[egui_snarl::InPin],
        _outputs: &[egui_snarl::OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }
}

fn color_pin(data: &Data) -> PinInfo {
    let color = match data {
        Data::Text(_) => Color32::from_rgb(16, 255, 16),
        Data::List(_) => Color32::from_rgb(16, 16, 255),
        Data::Error(_) => Color32::from_rgb(255, 16, 16),
    };
    PinInfo::circle().with_fill(color)
}
