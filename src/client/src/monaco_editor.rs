use web_sys::HtmlTextAreaElement;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Properties, PartialEq)]
pub struct MonacoEditorProps {
    pub initial_value: String,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub language: Option<String>,
    #[prop_or_default]
    pub theme: Option<String>,
    #[prop_or_default]
    pub readonly: bool,
}

pub struct MonacoEditor {
    editor_ref: NodeRef,
    monaco_ready: bool,
    fallback_value: String,
    editor_id: String,
}

pub enum MonacoMsg {
    MonacoReady,
    FallbackChanged(String),
}

impl Component for MonacoEditor {
    type Message = MonacoMsg;
    type Properties = MonacoEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let editor_id = format!("editor-container-{}", js_sys::Math::random());
        Self {
            editor_ref: NodeRef::default(),
            monaco_ready: false,
            fallback_value: String::new(),
            editor_id,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MonacoMsg::MonacoReady => {
                self.monaco_ready = true;
                true
            }
            MonacoMsg::FallbackChanged(value) => {
                self.fallback_value = value.clone();
                ctx.props().on_change.emit(value);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_fallback_input = ctx.link().callback(|e: InputEvent| {
            let target = e.target().unwrap();
            let input = target.dyn_into::<HtmlTextAreaElement>().unwrap();
            MonacoMsg::FallbackChanged(input.value())
        });

        html! {
            <div class="ggl-monaco-editor">
                <div id={self.editor_id.clone()} ref={self.editor_ref.clone()} style="height: 100%;"></div>
                // Fallback textarea (hidden when Monaco loads)
                <textarea
                    class={if self.monaco_ready { "ggl-monaco-fallback hidden" } else { "ggl-monaco-fallback" }}
                    value={ctx.props().initial_value.clone()}
                    oninput={on_fallback_input}
                    placeholder="Enter your GGL code here..."
                />
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.setup_monaco_editor(ctx);
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        // Update Monaco editor value if it changed externally
        if self.monaco_ready && ctx.props().initial_value != old_props.initial_value {
            let new_value = &ctx.props().initial_value;
            let editor_var = format!("monacoEditor_{}", self.editor_id.replace("-", "_"));
            let set_value_code = format!(
                "if (window['{}']) {{ window['{}'].setValue(`{}`); }}",
                editor_var,
                editor_var,
                new_value.replace('`', r#"\`"#).replace("${", r#"\${"#)
            );
            if let Err(e) = js_sys::eval(&set_value_code) {
                log(&format!("Error updating Monaco value: {:?}", e));
            }
        }
        false
    }
}

impl MonacoEditor {
    fn setup_monaco_editor(&mut self, ctx: &Context<Self>) {
        let initial_value = ctx.props().initial_value.clone();
        let language = ctx.props().language.clone().unwrap_or_else(|| "javascript".to_string());
        let theme = ctx.props().theme.clone().unwrap_or_else(|| "vs-dark".to_string());
        let readonly = ctx.props().readonly;
        let link = ctx.link().clone();
        let on_change = ctx.props().on_change.clone();
        let editor_id = self.editor_id.clone();
        let callback_name = format!("monacoChangeCallback_{}", editor_id.replace("-", "_"));

        // Setup Monaco using CDN
        let setup_code = format!(
            r#"
            require.config({{ paths: {{ vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.44.0/min/vs' }} }});
            require(['vs/editor/editor.main'], function(monaco) {{
                const container = document.getElementById('{}');
                if (container) {{
                    const editor = monaco.editor.create(container, {{
                        value: `{}`,
                        language: '{}',
                        theme: '{}',
                        fontSize: 14,
                        lineNumbers: 'on',
                        automaticLayout: true,
                        minimap: {{ enabled: false }},
                        scrollBeyondLastLine: false,
                        wordWrap: 'on',
                        readOnly: {}
                    }});

                    // Store editor reference
                    window['monacoEditor_{}'] = editor;

                    // Set up change listener
                    editor.onDidChangeModelContent(function() {{
                        const value = editor.getValue();
                        // Trigger change callback
                        if (window['{}']) {{
                            window['{}'](value);
                        }}
                    }});

                    console.log('Monaco Editor initialized: {}');
                }}
            }});
            "#,
            editor_id,
            initial_value.replace('`', r#"\`"#).replace("${", r#"\${"#),
            language,
            theme,
            if readonly { "true" } else { "false" },
            editor_id.replace("-", "_"),
            callback_name,
            callback_name,
            editor_id
        );

        // Set up global change callback
        let callback = Closure::wrap(Box::new(move |value: String| {
            on_change.emit(value);
        }) as Box<dyn FnMut(String)>);

        // Store callback reference globally with unique name
        let global = js_sys::global();
        js_sys::Reflect::set(&global, &callback_name.into(), callback.as_ref()).unwrap();
        callback.forget(); // Keep callback alive

        // Execute setup code
        if let Err(e) = js_sys::eval(&setup_code) {
            log(&format!("Error setting up Monaco: {:?}", e));
        }

        // Mark Monaco as ready
        link.send_message(MonacoMsg::MonacoReady);
    }

    /// Get the current value from the Monaco editor
    pub fn get_value() -> Option<String> {
        match js_sys::eval("window.monacoEditor ? window.monacoEditor.getValue() : null") {
            Ok(js_val) if !js_val.is_null() => js_val.as_string(),
            _ => None,
        }
    }

    /// Set the value in the Monaco editor
    pub fn set_value(value: &str) {
        let set_value_code = format!(
            "if (window.monacoEditor) {{ window.monacoEditor.setValue(`{}`); }}",
            value.replace('`', r#"\`"#).replace("${", r#"\${"#)
        );
        if let Err(e) = js_sys::eval(&set_value_code) {
            log(&format!("Error setting Monaco value: {:?}", e));
        }
    }
}
