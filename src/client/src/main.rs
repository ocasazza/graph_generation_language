mod monaco_editor;

use monaco_editor::MonacoEditor;
use yew::prelude::*;
use graph_generation_language::GGLEngine;

pub struct App {
    ggl_input: String,
    json_output: Option<Result<String, String>>,
}

pub enum Msg {
    EditorChanged(String),
    Generate,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            ggl_input: r#"graph social_network {
    // Define nodes with types and attributes
    node alice :person [name="Alice", age=30];
    node bob :person [name="Bob", age=25];

    // Create relationships
    edge friendship: alice -- bob [strength=0.8];

    // Generate additional structure
    generate complete {
        nodes: 3;
        prefix: "user";
    }
}"#.to_string(),
            json_output: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::EditorChanged(value) => {
                self.ggl_input = value;
                false
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_generate = ctx.link().callback(|_| Msg::Generate);
        let on_editor_change = ctx.link().callback(|value: String| Msg::EditorChanged(value));

        html! {
            <div class="ggl-editor-container">
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
                        {self.render_output()}
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
    fn render_output(&self) -> Html {
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
