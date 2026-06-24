<template>
  <div class="view active">
    <div class="section-header">
      <h2>Import / Export</h2>
    </div>
    <div class="settings-grid">
      <div class="setting-group">
        <h4>Export Configuration</h4>
        <p style="font-size:13px;color:var(--fg-muted)">
          Package your configs as a portable .forge file
        </p>
        <button class="btn btn-primary">Export as .forge</button>
      </div>
      <div class="setting-group">
        <h4>Import Configuration</h4>
        <p style="font-size:13px;color:var(--fg-muted)">
          Import configs from a .forge package or Git repo
        </p>
        <button class="btn btn-secondary">Import from Git Repository</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>
