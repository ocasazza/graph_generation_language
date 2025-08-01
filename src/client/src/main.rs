mod monaco_editor;
mod graph_visualizer;
#[cfg(test)]
mod example_tests;

use monaco_editor::MonacoEditor;
use graph_visualizer::{GraphVisualizerComponent, LayoutAlgorithm};
use yew::prelude::*;
use graph_generation_language::GGLEngine;
use wasm_bindgen::JsCast;

#[derive(Clone)]
pub struct GGLExample {
    name: &'static str,
    #[allow(dead_code)]
    description: &'static str,
    code: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OutputTab {
    Json,
    Visualization,
}

pub struct App {
    ggl_input: String,
    json_output: Option<Result<String, String>>,
    examples: Vec<GGLExample>,
    selected_example: usize,
    active_tab: OutputTab,
    // Visualization state
    layout_algorithm: LayoutAlgorithm,
    simulation_running: bool,
}

pub enum Msg {
    EditorChanged(String),
    Generate,
    ExampleSelected(usize),
    TabChanged(OutputTab),
    // Visualization messages
    LayoutChanged(LayoutAlgorithm),
    ToggleSimulation,
    ResetView,
}

fn load_examples() -> Vec<GGLExample> {
    vec![
        GGLExample {
            name: "Social Network",
            description: "Basic social network with friendship connections",
            code: include_str!("../examples/social_network.ggl"),
        },
        GGLExample {
            name: "HPC Network Design",
            description: "High-Performance Computing cluster with storage, management, and GPU fabrics",
            code: include_str!("../examples/hpc_network.ggl"),
        },
        GGLExample {
            name: "Chemical Reaction",
            description: "Simple chemical reaction with energy states and catalysis",
            code: include_str!("../examples/chemical_reaction.ggl"),
        },
        GGLExample {
            name: "Toroidal Mesh",
            description: "2D torus topology with wrap-around connections",
            code: include_str!("../examples/toroidal_mesh.ggl"),
        },
        GGLExample {
            name: "Protein Network",
            description: "Protein-protein interaction network for DNA damage response",
            code: include_str!("../examples/protein_network.ggl"),
        },
        GGLExample {
            name: "Quantum Circuit",
            description: "Quantum computing circuit with qubits and gates",
            code: include_str!("../examples/quantum_circuit.ggl"),
        },
        GGLExample {
            name: "Neural Network",
            description: "Multi-layer neural network with synaptic connections",
            code: include_str!("../examples/neural_network.ggl"),
        },
        GGLExample {
            name: "Crystal Lattice",
            description: "Diamond crystal structure with defects and dopants",
            code: include_str!("../examples/crystal_lattice.ggl"),
        },
    ]
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let examples = load_examples();
        let initial_code = examples[0].code.to_string();

        Self {
            ggl_input: initial_code,
            json_output: None,
            examples,
            selected_example: 0,
            active_tab: OutputTab::Json,
            layout_algorithm: LayoutAlgorithm::ForceDirected,
            simulation_running: true,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::EditorChanged(value) => {
                self.ggl_input = value;
                false
            }
            Msg::ExampleSelected(index) => {
                if index < self.examples.len() {
                    self.selected_example = index;
                    self.ggl_input = self.examples[index].code.to_string();
                }
                true
            }
            Msg::Generate => {
                // Get current value from Monaco editor if available, otherwise use stored value
                if let Some(current_value) = MonacoEditor::get_value() {
                    self.ggl_input = current_value;
                }

                let mut engine = GGLEngine::new();
                match engine.generate_from_ggl(&self.ggl_input) {
                    Ok(json) => {
                        // Pretty format the JSON
                        match serde_json::from_str::<serde_json::Value>(&json) {
                            Ok(parsed) => {
                                match serde_json::to_string_pretty(&parsed) {
                                    Ok(pretty) => self.json_output = Some(Ok(pretty)),
                                    Err(_) => self.json_output = Some(Ok(json)),
                                }
                            }
                            Err(_) => self.json_output = Some(Ok(json)),
                        }
                    }
                    Err(error) => {
                        self.json_output = Some(Err(error));
                    }
                }
                true
            }
            Msg::TabChanged(tab) => {
                self.active_tab = tab;
                true
            }
            Msg::LayoutChanged(layout) => {
                self.layout_algorithm = layout;
                true
            }
            Msg::ToggleSimulation => {
                self.simulation_running = !self.simulation_running;
                true
            }
            Msg::ResetView => {
                // Reset view will be handled by the visualizer component
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_generate = ctx.link().callback(|_| Msg::Generate);
        let on_editor_change = ctx.link().callback(|value: String| Msg::EditorChanged(value));
        let on_example_change = ctx.link().callback(|e: Event| {
            let target = e.target().unwrap();
            let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let index = select.selected_index() as usize;
            Msg::ExampleSelected(index)
        });

        html! {
            <div class="ggl-editor-container">
                <div class="example-dropdown-container">
                    <div class="example-dropdown-left">
                        <label for="example-select">{"Select Example: "}</label>
                        <select id="example-select" onchange={on_example_change} value={self.selected_example.to_string()}>
                            { for self.examples.iter().enumerate().map(|(index, example)| {
                                html! {
                                    <option value={index.to_string()} selected={index == self.selected_example}>
                                        {example.name}
                                    </option>
                                }
                            })}
                        </select>
                    </div>
                    <div class="example-dropdown-right">
                        {if self.active_tab == OutputTab::Visualization {
                            html! {
                                <>
                                    <label>{"Layout: "}</label>
                                    <select onchange={ctx.link().callback(|e: Event| {
                                        let target = e.target().unwrap();
                                        let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                                        let value = select.value();
                                        match value.as_str() {
                                            "Circle" => Msg::LayoutChanged(LayoutAlgorithm::Circle),
                                            "Grid" => Msg::LayoutChanged(LayoutAlgorithm::Grid),
                                            "Random" => Msg::LayoutChanged(LayoutAlgorithm::Random),
                                            _ => Msg::LayoutChanged(LayoutAlgorithm::ForceDirected),
                                        }
                                    })}>
                                        <option value="ForceDirected" selected={self.layout_algorithm == LayoutAlgorithm::ForceDirected}>{"Force Directed"}</option>
                                        <option value="Circle" selected={self.layout_algorithm == LayoutAlgorithm::Circle}>{"Circle"}</option>
                                        <option value="Grid" selected={self.layout_algorithm == LayoutAlgorithm::Grid}>{"Grid"}</option>
                                        <option value="Random" selected={self.layout_algorithm == LayoutAlgorithm::Random}>{"Random"}</option>
                                    </select>

                                    {if self.layout_algorithm == LayoutAlgorithm::ForceDirected {
                                        html! {
                                            <button onclick={ctx.link().callback(|_| Msg::ToggleSimulation)}>
                                                {if self.simulation_running { "⏸ Pause" } else { "▶ Play" }}
                                            </button>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    <button onclick={ctx.link().callback(|_| Msg::ResetView)}>{"🔄 Reset View"}</button>
                                </>
                            }
                        } else {
                            html! {}
                        }}

                        <button
                            class={if self.active_tab == OutputTab::Json { "tab-button active" } else { "tab-button" }}
                            onclick={ctx.link().callback(|_| Msg::TabChanged(OutputTab::Json))}
                        >
                            {"📄 JSON"}
                        </button>
                        <button
                            class={if self.active_tab == OutputTab::Visualization { "tab-button active" } else { "tab-button" }}
                            onclick={ctx.link().callback(|_| Msg::TabChanged(OutputTab::Visualization))}
                        >
                            {"🎨 Visualization"}
                        </button>
                    </div>
                </div>
                <div class="ggl-editor-layout">
                    // Left panel - Editor wrapper
                    <div class="ggl-editor-panel">
                        <MonacoEditor
                            initial_value={self.ggl_input.clone()}
                            on_change={on_editor_change}
                            language="null"
                            theme="vs-dark"
                        />
                    </div>
                    <div style="height: 100%; width: 12px;"></div>
                    // Right panel - Output wrapper
                    <div class="ggl-output-panel">
                        {self.render_output(ctx)}
                    </div>
                </div>
                <button class="generate-btn" onclick={on_generate}>
                    {"🔄 Generate Graph"}
                </button>
            </div>
        }
    }
}

impl App {
    fn render_output(&self, _ctx: &Context<Self>) -> Html {
        match self.active_tab {
            OutputTab::Json => self.render_json_output(),
            OutputTab::Visualization => self.render_visualization(),
        }
    }

    fn render_json_output(&self) -> Html {
        let output_content = match &self.json_output {
            Some(Ok(json)) => json.clone(),
            Some(Err(error)) => format!("// Error:\n{error}"),
            None => "// Click 'Generate Graph' to process your GGL code...".to_string(),
        };

        html! {
            <MonacoEditor
                initial_value={output_content}
                on_change={Callback::noop()}  // Read-only, no changes needed
                language="json"
                theme="vs-dark"
                readonly=true
            />
        }
    }

    fn render_visualization(&self) -> Html {
        let graph_json = match &self.json_output {
            Some(Ok(json)) => Some(json.clone()),
            _ => None,
        };

        html! {
            <GraphVisualizerComponent
                graph_json={graph_json}
                layout_algorithm={self.layout_algorithm}
                simulation_running={self.simulation_running}
                reset_view={false}
            />
        }
    }
}

fn main() {
    let app_element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("app")
        .expect("Failed to find #app element");

    yew::Renderer::<App>::with_root(app_element).render();
}
