<template>
  <div class="editor-container">
    <!-- Main content area with two sides -->
    <div class="header">
      <h3>Graph Generation Language (GGL) Browser Demo</h3>
    </div>
    <div class="main-content">
      <!-- Left side - Editor -->
      <div class="left-panel">
        <div class="editor-wrapper">
          <div ref="editorContainer" class="monaco-editor"></div>
        </div>
      </div>

      <!-- Right side - Output/Preview -->
      <div class="right-panel">
        <div class="editor-wrapper">
          <div ref="outputEditorContainer" class="monaco-editor"></div>
        </div>
      </div>
    </div>

    <!-- Status bar at the bottom -->
    <div class="status-bar">
      <div class="status-left">
        <select v-model="selectedExample" @change="changeExample" class="example-select">
          <option v-for="exampleName in exampleNames" :key="exampleName" :value="exampleName">
            {{ formatExampleName(exampleName) }}
          </option>
        </select>
        <button @click="generateGraph" class="status-btn status-btn-primary">Generate</button>
        <button @click="clearEditor" class="status-btn">Clear</button>
      </div>
      <div v-if="generationTime" class="status-right">
        <div class="status-btn">Graph generated in {{ generationTime }} ms.</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue';
import * as monaco from 'monaco-editor';
import { examples } from './examples.js';

// Reactive data
const selectedExample = ref('neuralNetwork');
const output = ref('Ready to generate graph...');
const generationTime = ref(null);
const editorContainer = ref(null);
const outputEditorContainer = ref(null);
let editor = null;
let outputEditor = null;
let GGL = null;

// Get example names for dropdown
const exampleNames = Object.keys(examples);

// Format example name for display
const formatExampleName = (name) => {
  return name.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase());
};

// Initialize GGL WASM module
const initializeGGL = async () => {
  try {
    const init = await import('./pkg/ggl_wasm.js').then(m => m.default);
    GGL = await import('./pkg/ggl_wasm.js');
    await init();
    if (outputEditor) {
      outputEditor.setValue('GGL WASM module loaded successfully. Ready to generate graphs!');
    }
  } catch (error) {
    const errorMsg = `Error loading GGL WASM module: ${error.message}`;
    if (outputEditor) {
      outputEditor.setValue(errorMsg);
    }
    console.error('Failed to load GGL WASM:', error);
  }
};

// Initialize Monaco Editor
const initializeEditor = async () => {
  await nextTick();

  if (editorContainer.value && !editor) {
    editor = monaco.editor.create(editorContainer.value, {
      value: examples[selectedExample.value] || '// Select an example from the dropdown',
      language: 'text', // GGL language, fallback to text for now
      theme: 'hc-black',
      fontSize: 14,
      wordWrap: 'on',
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      lineNumbers: 'on',
      roundedSelection: false,
      readOnly: false,
      cursorStyle: 'line',
    });

    // Listen for content changes
    editor.onDidChangeModelContent(() => {
      // You can add auto-save or other logic here
    });
  }

  // Initialize output editor
  if (outputEditorContainer.value && !outputEditor) {
    outputEditor = monaco.editor.create(outputEditorContainer.value, {
      value: 'Ready to generate graph...',
      language: 'json',
      theme: 'hc-black',
      fontSize: 14,
      wordWrap: 'on',
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      lineNumbers: 'on',
      roundedSelection: false,
      readOnly: true,
      cursorStyle: 'line',
    });
  }
};

// Generate graph using GGL WASM
const generateGraph = async () => {
  if (!editor) {
    if (outputEditor) {
      outputEditor.setValue('Editor not initialized');
    }
    return;
  }

  if (!GGL) {
    if (outputEditor) {
      outputEditor.setValue('GGL WASM module not loaded. Please refresh and try again.');
    }
    return;
  }

  try {
    const gglCode = editor.getValue();

    if (!gglCode.trim()) {
      if (outputEditor) {
        outputEditor.setValue('Please enter some GGL code to generate a graph.');
      }
      return;
    }

    if (outputEditor) {
      outputEditor.setValue('Generating graph...');
    }

    // Start timer
    const startTime = performance.now();

    const result = GGL.parse_ggl(gglCode);

    // End timer and calculate elapsed time
    const endTime = performance.now();
    const elapsedTime = Math.round(endTime - startTime);
    generationTime.value = elapsedTime;

    if (outputEditor) {
      // Format the JSON result properly
      const formattedJson = JSON.stringify(JSON.parse(result), null, 2);
      outputEditor.setValue(formattedJson);
    }

  } catch (error) {
    // Clear generation time on error
    generationTime.value = null;
    if (outputEditor) {
      outputEditor.setValue(`Error generating graph:\n${error.message}`);
    }
    console.error('GGL parsing error:', error);
  }
};

// Clear the editor
const clearEditor = () => {
  if (editor) {
    editor.setValue('');
  }
  if (outputEditor) {
    outputEditor.setValue('Editor cleared. Ready for new GGL code...');
  }
};

// Change example
const changeExample = () => {
  if (editor && examples[selectedExample.value]) {
    editor.setValue(examples[selectedExample.value]);
  }
  if (outputEditor) {
    outputEditor.setValue(`Switched to ${formatExampleName(selectedExample.value)} example.`);
  }
};

// Component lifecycle
onMounted(async () => {
  await initializeEditor();
  await initializeGGL();
});
</script>

<style>

html, body * {
  color: wheat;
  font-family: 'Fira Code', monospace;
}

.editor-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.status-bar {
  height: 24px;
  background: black;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  flex-shrink: 0;
  font-size: 12px;
  border: 1px solid wheat;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.example-select {
  background: transparent;
  border: none;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 3px;
  transition: background-color 0.2s ease;
}

.example-select:hover {
  background: rgba(255, 255, 255, 0.1);
}

.example-select:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.1);
}

.status-btn {
  background: transparent;
  border: none;
  font-size: 12px;
  cursor: pointer;
  padding: 3px 8px;
  border-radius: 3px;
  transition: background-color 0.2s ease;
}

.status-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.status-btn-primary {
  background: rgba(255, 255, 255, 0.1);
  font-weight: 500;
}

.status-btn-primary:hover {
  background: rgba(255, 255, 255, 0.2);
}

.main-content {
  border: 1px solid wheat;
  flex: 1;
  display: flex;
  overflow: hidden;
}

.left-panel,
.right-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.left-panel {
  border-right: 1px solid #3e3e42;
}

.header {
  border: 1px solid wheat;
  height: 30px;
  background: black;
  display: flex;
  align-items: center;
  padding: 0 16px;
  flex-shrink: 0;
  color: wheat;
}

.panel-header {
  height: 20px;
  background: black;
  display: flex;
  align-items: center;
  padding: 0 16px;
  flex-shrink: 0;
}

.panel-header h3 {
  margin: 0;
  color: #cccccc;
  font-size: 14px;
  font-weight: 600;
}

.editor-wrapper {
  flex: 1;
  overflow: hidden;
}

.monaco-editor {
  height: 100%;
  width: 100%;
}

.output-wrapper {
  flex: 1;
  overflow: auto;
  background: #1e1e1e;
}

.output-content {
  margin: 0;
  padding: 16px;
  color: #d4d4d4;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* Scrollbar styling for dark theme */
.output-wrapper::-webkit-scrollbar {
  width: 14px;
}

.output-wrapper::-webkit-scrollbar-track {
  background: #1e1e1e;
}

.output-wrapper::-webkit-scrollbar-thumb {
  background: #424242;
  border-radius: 7px;
}

.output-wrapper::-webkit-scrollbar-thumb:hover {
  background: #4f4f4f;
}
</style>
