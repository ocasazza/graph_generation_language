//! Graph visualization component using HTML5 Canvas
//!
//! This module provides an interactive graph visualization that can render
//! graphs generated from GGL code. It supports pan, zoom, node selection,
//! and multiple layout algorithms using HTML5 Canvas API.

use yew::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};
use wasm_bindgen::{JsCast, JsValue};
use graph_generation_language::types::{Graph, Node, Edge};
use serde_json::Value;
use std::collections::HashMap;
use gloo::timers::callback::Interval;

/// 2D position
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos2 {
    pub x: f32,
    pub y: f32,
}

impl Pos2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: Pos2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl std::ops::Add<Vec2> for Pos2 {
    type Output = Pos2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Pos2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Pos2 {
    type Output = Vec2;
    fn sub(self, rhs: Pos2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// 2D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2::new(self.x / len, self.y / len)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    pub fn min_elem(&self) -> f32 {
        self.x.min(self.y)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl std::ops::Mul<f32> for Pos2 {
    type Output = Pos2;
    fn mul(self, rhs: f32) -> Self::Output {
        Pos2::new(self.x * rhs, self.y * rhs)
    }
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const LIGHT_BLUE: Color = Color { r: 173, g: 216, b: 230, a: 255 };
    pub const LIGHT_GREEN: Color = Color { r: 144, g: 238, b: 144, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const ORANGE: Color = Color { r: 255, g: 165, b: 0, a: 255 };
    pub const LIGHT_GRAY: Color = Color { r: 211, g: 211, b: 211, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };

    pub fn to_css_string(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a as f32 / 255.0)
    }
}

/// Represents a visual node in the graph
#[derive(Debug, Clone)]
pub struct VisualNode {
    pub id: String,
    pub position: Pos2,
    pub velocity: Vec2,
    pub radius: f32,
    pub color: Color,
    pub label: String,
    pub metadata: HashMap<String, Value>,
    pub selected: bool,
}

/// Represents a visual edge in the graph
#[derive(Debug, Clone)]
pub struct VisualEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub directed: bool,
    pub color: Color,
    pub width: f32,
    pub metadata: HashMap<String, Value>,
}

/// Layout algorithms for graph positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutAlgorithm {
    ForceDirected,
    Circle,
    Grid,
    Random,
}

impl std::fmt::Display for LayoutAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutAlgorithm::ForceDirected => write!(f, "Force Directed"),
            LayoutAlgorithm::Circle => write!(f, "Circle"),
            LayoutAlgorithm::Grid => write!(f, "Grid"),
            LayoutAlgorithm::Random => write!(f, "Random"),
        }
    }
}

/// Configuration for force-directed layout
#[derive(Debug, Clone)]
pub struct ForceConfig {
    pub spring_strength: f32,
    pub spring_length: f32,
    pub repulsion_strength: f32,
    pub damping: f32,
    pub center_strength: f32,
}

impl Default for ForceConfig {
    fn default() -> Self {
        Self {
            spring_strength: 0.1,
            spring_length: 50.0,
            repulsion_strength: 1000.0,
            damping: 0.9,
            center_strength: 0.01,
        }
    }
}

/// Graph visualization data
pub struct GraphVisualizerData {
    nodes: HashMap<String, VisualNode>,
    edges: Vec<VisualEdge>,
    layout: LayoutAlgorithm,
    canvas_size: Vec2,
    camera_offset: Vec2,
    zoom: f32,
    selected_node: Option<String>,
    simulation_running: bool,
    force_config: ForceConfig,
}

impl Default for GraphVisualizerData {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            layout: LayoutAlgorithm::ForceDirected,
            canvas_size: Vec2::new(800.0, 600.0),
            camera_offset: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            selected_node: None,
            simulation_running: true,
            force_config: ForceConfig::default(),
        }
    }
}

/// Messages for the graph visualizer component
pub enum GraphVisualizerMsg {
    Render,
    CanvasClick(MouseEvent),
    LayoutChanged(LayoutAlgorithm),
    ToggleSimulation,
    ResetView,
}

/// Props for the graph visualizer component
#[derive(Properties, PartialEq)]
pub struct GraphVisualizerProps {
    pub graph_json: Option<String>,
    pub layout_algorithm: LayoutAlgorithm,
    pub simulation_running: bool,
    pub reset_view: bool,
}

/// Main graph visualization component
pub struct GraphVisualizerComponent {
    canvas_ref: NodeRef,
    data: GraphVisualizerData,
    _render_interval: Option<Interval>,
}

impl Component for GraphVisualizerComponent {
    type Message = GraphVisualizerMsg;
    type Properties = GraphVisualizerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let render_interval = {
            let link = ctx.link().clone();
            Some(Interval::new(16, move || { // ~60fps
                link.send_message(GraphVisualizerMsg::Render);
            }))
        };

        Self {
            canvas_ref: NodeRef::default(),
            data: GraphVisualizerData::default(),
            _render_interval: render_interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GraphVisualizerMsg::Render => {
                self.update_simulation();
                self.render_canvas();
                false
            }
            GraphVisualizerMsg::CanvasClick(event) => {
                self.handle_canvas_click(event);
                true
            }
            GraphVisualizerMsg::LayoutChanged(layout) => {
                self.data.layout = layout;
                self.apply_layout();
                true
            }
            GraphVisualizerMsg::ToggleSimulation => {
                self.data.simulation_running = !self.data.simulation_running;
                true
            }
            GraphVisualizerMsg::ResetView => {
                self.data.zoom = 1.0;
                self.data.camera_offset = Vec2::new(0.0, 0.0);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        let mut changed = false;

        // Handle graph data changes
        if let Some(graph_json) = &props.graph_json {
            if let Err(e) = self.load_graph(graph_json) {
                web_sys::console::error_1(&format!("Failed to load graph: {}", e).into());
            }
            changed = true;
        }

        // Handle layout algorithm changes
        if props.layout_algorithm != old_props.layout_algorithm {
            self.data.layout = props.layout_algorithm;
            self.apply_layout();
            changed = true;
        }

        // Handle simulation state changes
        if props.simulation_running != old_props.simulation_running {
            self.data.simulation_running = props.simulation_running;
            changed = true;
        }

        // Handle reset view
        if props.reset_view && !old_props.reset_view {
            self.data.zoom = 1.0;
            self.data.camera_offset = Vec2::new(0.0, 0.0);
            changed = true;
        }

        changed
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_canvas_click = ctx.link().callback(GraphVisualizerMsg::CanvasClick);

        html! {
            <div class="graph-visualizer">
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="800"
                    height="600"
                    onclick={on_canvas_click}
                    style="border: 1px solid #ccc; cursor: pointer; width: 100%; height: 100%;"
                />

                {if let Some(selected_id) = &self.data.selected_node {
                    if let Some(node) = self.data.nodes.get(selected_id) {
                        html! {
                            <div class="node-info">
                                <h4>{"Selected Node"}</h4>
                                <p><strong>{"ID: "}</strong>{&node.id}</p>
                                <p><strong>{"Label: "}</strong>{&node.label}</p>
                                {if !node.metadata.is_empty() {
                                    html! {
                                        <div>
                                            <strong>{"Metadata:"}</strong>
                                            <pre>{serde_json::to_string_pretty(&node.metadata).unwrap_or_default()}</pre>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }
}

impl GraphVisualizerComponent {
    /// Load graph data from JSON string
    fn load_graph(&mut self, json_data: &str) -> Result<(), String> {
        let graph: Graph = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse graph JSON: {}", e))?;

        self.load_graph_struct(&graph);
        Ok(())
    }

    /// Load graph data from Graph struct
    fn load_graph_struct(&mut self, graph: &Graph) {
        self.data.nodes.clear();
        self.data.edges.clear();

        // Convert nodes
        let canvas_size = self.data.canvas_size;
        for (idx, (id, node)) in graph.nodes.iter().enumerate() {
            // Generate deterministic position based on index
            let x = ((idx * 13 + 31) % 1000) as f32 * canvas_size.x / 1000.0;
            let y = ((idx * 19 + 47) % 1000) as f32 * canvas_size.y / 1000.0;

            let visual_node = VisualNode {
                id: id.clone(),
                position: Pos2::new(x, y),
                velocity: Vec2::new(0.0, 0.0),
                radius: 10.0,
                color: self.node_color(&node.r#type),
                label: if node.r#type.is_empty() { id.clone() } else { node.r#type.clone() },
                metadata: node.metadata.clone(),
                selected: false,
            };
            self.data.nodes.insert(id.clone(), visual_node);
        }

        // Convert edges
        for (id, edge) in &graph.edges {
            let visual_edge = VisualEdge {
                id: id.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                directed: edge.directed,
                color: Color::GRAY,
                width: 2.0,
                metadata: edge.metadata.clone(),
            };
            self.data.edges.push(visual_edge);
        }

        // Apply initial layout
        self.apply_layout();
        self.data.simulation_running = true;
    }

    /// Generate a random position within the canvas
    fn random_position(&self) -> Pos2 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::ptr::addr_of!(self).hash(&mut hasher);
        let seed = hasher.finish();

        let x = (seed % 1000) as f32 * self.data.canvas_size.x / 1000.0;
        let y = ((seed / 1000) % 1000) as f32 * self.data.canvas_size.y / 1000.0;

        Pos2::new(x, y)
    }

    /// Get color based on node type
    fn node_color(&self, node_type: &str) -> Color {
        match node_type {
            "person" | "user" => Color::LIGHT_BLUE,
            "server" | "service" => Color::LIGHT_GREEN,
            "database" | "storage" => Color::YELLOW,
            "network" | "connection" => Color::ORANGE,
            _ => Color::LIGHT_GRAY,
        }
    }

    /// Apply the selected layout algorithm
    fn apply_layout(&mut self) {
        match self.data.layout {
            LayoutAlgorithm::Circle => self.apply_circle_layout(),
            LayoutAlgorithm::Grid => self.apply_grid_layout(),
            LayoutAlgorithm::Random => self.apply_random_layout(),
            LayoutAlgorithm::ForceDirected => {
                // Force-directed layout is applied continuously in update
            }
        }
    }

    /// Apply circular layout
    fn apply_circle_layout(&mut self) {
        let center = self.data.canvas_size * 0.5;
        let radius = self.data.canvas_size.min_elem() * 0.3;
        let count = self.data.nodes.len();

        for (i, node) in self.data.nodes.values_mut().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / count as f32;
            node.position = Pos2::new(center.x, center.y) + Vec2::new(
                radius * angle.cos(),
                radius * angle.sin(),
            );
            node.velocity = Vec2::new(0.0, 0.0);
        }
    }

    /// Apply grid layout
    fn apply_grid_layout(&mut self) {
        let count = self.data.nodes.len();
        let cols = (count as f32).sqrt().ceil() as usize;
        let cell_size = Vec2::new(
            self.data.canvas_size.x / cols as f32,
            self.data.canvas_size.y / ((count + cols - 1) / cols) as f32,
        );

        for (i, node) in self.data.nodes.values_mut().enumerate() {
            let row = i / cols;
            let col = i % cols;
            node.position = Pos2::new(
                (col as f32 + 0.5) * cell_size.x,
                (row as f32 + 0.5) * cell_size.y,
            );
            node.velocity = Vec2::new(0.0, 0.0);
        }
    }

    /// Apply random layout
    fn apply_random_layout(&mut self) {
        let canvas_size = self.data.canvas_size;
        for (i, node) in self.data.nodes.values_mut().enumerate() {
            // Generate deterministic random position based on index
            let x = ((i * 17 + 42) % 1000) as f32 * canvas_size.x / 1000.0;
            let y = ((i * 23 + 67) % 1000) as f32 * canvas_size.y / 1000.0;
            node.position = Pos2::new(x, y);
            node.velocity = Vec2::new(0.0, 0.0);
        }
    }

    /// Update force-directed simulation
    fn update_simulation(&mut self) {
        if !self.data.simulation_running || self.data.layout != LayoutAlgorithm::ForceDirected {
            return;
        }

        let center = self.data.canvas_size * 0.5;
        let mut forces: HashMap<String, Vec2> = HashMap::new();

        // Initialize forces
        for id in self.data.nodes.keys() {
            forces.insert(id.clone(), Vec2::new(0.0, 0.0));
        }

        // Repulsion forces between all nodes
        let node_ids: Vec<String> = self.data.nodes.keys().cloned().collect();
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let id1 = &node_ids[i];
                let id2 = &node_ids[j];

                if let (Some(node1), Some(node2)) = (self.data.nodes.get(id1), self.data.nodes.get(id2)) {
                    let delta = node1.position - node2.position;
                    let distance = delta.length().max(1.0);
                    let force_magnitude = self.data.force_config.repulsion_strength / (distance * distance);
                    let force = delta.normalized() * force_magnitude;

                    *forces.get_mut(id1).unwrap() += force;
                    *forces.get_mut(id2).unwrap() -= force;
                }
            }
        }

        // Spring forces for connected nodes
        for edge in &self.data.edges {
            if let (Some(source), Some(target)) = (
                self.data.nodes.get(&edge.source),
                self.data.nodes.get(&edge.target),
            ) {
                let delta = target.position - source.position;
                let distance = delta.length();
                let spring_force = self.data.force_config.spring_strength *
                    (distance - self.data.force_config.spring_length);
                let force = delta.normalized() * spring_force;

                *forces.get_mut(&edge.source).unwrap() += force;
                *forces.get_mut(&edge.target).unwrap() -= force;
            }
        }

        // Center attraction
        for (id, node) in &self.data.nodes {
            let to_center = Pos2::new(center.x, center.y) - node.position;
            let center_force = to_center * self.data.force_config.center_strength;
            *forces.get_mut(id).unwrap() += center_force;
        }

        // Apply forces and update positions
        for (id, force) in forces {
            if let Some(node) = self.data.nodes.get_mut(&id) {
                node.velocity += force;
                node.velocity *= self.data.force_config.damping;
                node.position = node.position + node.velocity;
            }
        }
    }

    /// Handle canvas click events
    fn handle_canvas_click(&mut self, event: MouseEvent) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
        let rect = canvas.get_bounding_client_rect();
        let click_pos = Pos2::new(
            event.client_x() as f32 - rect.left() as f32,
            event.client_y() as f32 - rect.top() as f32,
        );

        // Check if we clicked on a node - collect screen positions first
        let zoom = self.data.zoom;
        let camera_offset = self.data.camera_offset;
        let mut clicked_node: Option<String> = None;

        for (id, node) in &self.data.nodes {
            let screen_pos = (node.position + camera_offset) * zoom;
            let distance = click_pos.distance_to(screen_pos);
            if distance <= node.radius * zoom {
                clicked_node = Some(id.clone());
                break;
            }
        }

        // Update selection state
        self.data.selected_node = clicked_node.clone();
        for (id, node) in &mut self.data.nodes {
            node.selected = clicked_node.as_ref() == Some(id);
        }
    }

    /// Render the graph on canvas
    fn render_canvas(&self) {
        if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
            if let Ok(context) = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
            {
                // Clear canvas
                context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

                // Render edges
                for edge in &self.data.edges {
                    if let (Some(source), Some(target)) = (
                        self.data.nodes.get(&edge.source),
                        self.data.nodes.get(&edge.target),
                    ) {
                        self.draw_edge(&context, edge, source.position, target.position);
                    }
                }

                // Render nodes
                for node in self.data.nodes.values() {
                    self.draw_node(&context, node);
                }
            }
        }
    }

    /// Draw an edge on the canvas
    fn draw_edge(&self, context: &CanvasRenderingContext2d, edge: &VisualEdge, start_pos: Pos2, end_pos: Pos2) {
        let start = self.world_to_screen(start_pos);
        let end = self.world_to_screen(end_pos);

        context.begin_path();
        context.set_stroke_style(&JsValue::from(edge.color.to_css_string()));
        context.set_line_width(edge.width as f64);
        context.move_to(start.x as f64, start.y as f64);
        context.line_to(end.x as f64, end.y as f64);
        context.stroke();

        // Draw arrow for directed edges
        if edge.directed {
            self.draw_arrow(context, start, end, &edge.color);
        }
    }

    /// Draw a node on the canvas
    fn draw_node(&self, context: &CanvasRenderingContext2d, node: &VisualNode) {
        let screen_pos = self.world_to_screen(node.position);
        let radius = node.radius * self.data.zoom;

        let color = if node.selected {
            Color::WHITE
        } else {
            node.color
        };

        // Draw node circle
        context.begin_path();
        context.set_fill_style(&JsValue::from(color.to_css_string()));
        context.arc(
            screen_pos.x as f64,
            screen_pos.y as f64,
            radius as f64,
            0.0,
            2.0 * std::f64::consts::PI,
        ).unwrap();
        context.fill();

        // Draw node border
        context.begin_path();
        context.set_stroke_style(&JsValue::from(Color::BLACK.to_css_string()));
        context.set_line_width(2.0);
        context.arc(
            screen_pos.x as f64,
            screen_pos.y as f64,
            radius as f64,
            0.0,
            2.0 * std::f64::consts::PI,
        ).unwrap();
        context.stroke();

        // Draw label
        if self.data.zoom > 0.5 {
            context.set_fill_style(&JsValue::from(Color::BLACK.to_css_string()));
            context.set_font("12px Arial");
            context.set_text_align("center");
            context.fill_text(
                &node.label,
                screen_pos.x as f64,
                (screen_pos.y + radius + 15.0) as f64,
            ).unwrap();
        }
    }

    /// Draw an arrow for directed edges
    fn draw_arrow(&self, context: &CanvasRenderingContext2d, start: Pos2, end: Pos2, color: &Color) {
        let direction = (end - start).normalized();
        let arrow_length = 10.0 * self.data.zoom;
        let arrow_angle = 0.5;

        let arrow_tip = end + direction * (-15.0 * self.data.zoom); // Offset from node
        let left = arrow_tip + direction.rotate(arrow_angle) * (-arrow_length);
        let right = arrow_tip + direction.rotate(-arrow_angle) * (-arrow_length);

        context.begin_path();
        context.set_stroke_style(&JsValue::from(color.to_css_string()));
        context.set_line_width(2.0);
        context.move_to(arrow_tip.x as f64, arrow_tip.y as f64);
        context.line_to(left.x as f64, left.y as f64);
        context.move_to(arrow_tip.x as f64, arrow_tip.y as f64);
        context.line_to(right.x as f64, right.y as f64);
        context.stroke();
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, world_pos: Pos2) -> Pos2 {
        let transformed = (world_pos + self.data.camera_offset) * self.data.zoom;
        transformed
    }
}

// Helper trait for Vec2 rotation
trait Rotate {
    fn rotate(self, angle: f32) -> Self;
}

impl Rotate for Vec2 {
    fn rotate(self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec2::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }
}
