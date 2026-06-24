<template>
  <div class="groups-panel">
    <div class="panel-header">
      <h4>Groups</h4>
      <button
        class="btn-icon"
        @click="startCreating"
        title="Add Group"
        :disabled="isCreating"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
    </div>

    <!-- All Servers (default) -->
    <button
      class="group-item"
      :class="{ active: activeGroupId === null }"
      @click="$emit('select', null)"
    >
      <span class="group-dot" :style="{ background: '#71717A' }"></span>
      <span class="group-name">All Servers</span>
      <span class="group-count">{{ totalCount }}</span>
    </button>

    <!-- Create form -->
    <div v-if="isCreating" class="group-create-form">
      <input
        ref="newGroupInput"
        v-model="newGroupName"
        type="text"
        placeholder="Group name"
        @keyup.enter="handleCreate"
        @keyup.escape="cancelCreating"
        maxlength="32"
      />
      <div class="color-picker">
        <button
          v-for="color in colorPresets"
          :key="color"
          class="color-option"
          :class="{ selected: newGroupColor === color }"
          :style="{ background: color }"
          @click="newGroupColor = color"
        ></button>
      </div>
      <div class="form-actions">
        <button class="btn btn-sm btn-secondary" @click="cancelCreating">Cancel</button>
        <button
          class="btn btn-sm btn-primary"
          :disabled="!newGroupName.trim()"
          @click="handleCreate"
        >
          Create
        </button>
      </div>
    </div>

    <!-- Group List -->
    <div class="group-list">
      <div
        v-for="group in groups"
        :key="group.id"
        class="group-item-wrapper"
      >
        <!-- View mode -->
        <div
          v-if="editingGroupId !== group.id"
          class="group-item"
          :class="{ active: activeGroupId === group.id, hidden: !group.isVisible }"
        >
          <button
            class="group-select"
            @click="$emit('select', group.id)"
          >
            <span class="group-dot" :style="{ background: group.color }"></span>
            <span class="group-name">{{ group.name }}</span>
            <span class="group-count">{{ getGroupServerCount(group.id) }}</span>
          </button>

          <div class="group-actions">
            <button
              class="action-btn"
              @click.stop="startEditing(group)"
              title="Edit group"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
            </button>
            <button
              class="action-btn"
              @click.stop="$emit('toggle-visibility', group.id)"
              :title="group.isVisible ? 'Hide group' : 'Show group'"
            >
              <svg v-if="group.isVisible" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
            </button>
            <button
              class="action-btn danger"
              @click.stop="handleDelete(group.id)"
              title="Delete group"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Edit mode -->
        <div v-else class="group-edit-form">
          <input
            v-model="editingGroupName"
            type="text"
            placeholder="Group name"
            @keyup.enter="saveEdit(group)"
            @keyup.escape="cancelEditing"
            maxlength="32"
          />
          <div class="color-picker">
            <button
              v-for="color in colorPresets"
              :key="color"
              class="color-option"
              :class="{ selected: editingGroupColor === color }"
              :style="{ background: color }"
              @click="editingGroupColor = color"
            ></button>
          </div>
          <div class="form-actions">
            <button class="btn btn-sm btn-secondary" @click="cancelEditing">Cancel</button>
            <button
              class="btn btn-sm btn-primary"
              :disabled="!editingGroupName.trim()"
              @click="saveEdit(group)"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="groups.length === 0 && !isCreating" class="empty-state">
      <p>No groups yet</p>
      <button class="btn btn-sm btn-secondary" @click="startCreating">
        Create your first group
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
import type { MCPGroup } from '@/types';
import { confirm } from '@/utils/dialog';

interface Props {
  groups: MCPGroup[];
  activeGroupId: string | null;
  totalCount?: number;
}

const props = withDefaults(defineProps<Props>(), {
  totalCount: 0,
});

const emit = defineEmits<{
  (e: 'select', groupId: string | null): void;
  (e: 'create', data: { name: string; color: string }): void;
  (e: 'update', group: MCPGroup): void;
  (e: 'delete', groupId: string): void;
  (e: 'toggle-visibility', groupId: string): void;
}>();

// Color presets (Forge-aligned)
const colorPresets = [
  '#F59E0B', // Amber (default)
  '#10B981', // Emerald
  '#EF4444', // Red
  '#3B82F6', // Blue
  '#8B5CF6', // Violet
  '#EC4899', // Pink
  '#06B6D4', // Cyan
  '#F97316', // Orange
];

// State
const isCreating = ref(false);
const editingGroupId = ref<string | null>(null);
const newGroupName = ref('');
const newGroupColor = ref('#F59E0B');
const editingGroupName = ref('');
const editingGroupColor = ref('#F59E0B');

function startCreating() {
  isCreating.value = true;
  newGroupName.value = '';
  newGroupColor.value = '#F59E0B';
  nextTick(() => {
    const input = document.querySelector('.group-create-form input') as HTMLInputElement;
    input?.focus();
  });
}

function cancelCreating() {
  isCreating.value = false;
  newGroupName.value = '';
}

function handleCreate() {
  if (!newGroupName.value.trim()) return;

  emit('create', {
    name: newGroupName.value.trim(),
    color: newGroupColor.value,
  });

  cancelCreating();
}

function startEditing(group: MCPGroup) {
  editingGroupId.value = group.id;
  editingGroupName.value = group.name;
  editingGroupColor.value = group.color;
  nextTick(() => {
    const input = document.querySelector('.group-edit-form input') as HTMLInputElement;
    input?.focus();
  });
}

function cancelEditing() {
  editingGroupId.value = null;
  editingGroupName.value = '';
}

function saveEdit(group: MCPGroup) {
  if (!editingGroupName.value.trim()) return;

  emit('update', {
    ...group,
    name: editingGroupName.value.trim(),
    color: editingGroupColor.value,
  });

  cancelEditing();
}

async function handleDelete(groupId: string) {
  if (await confirm('Delete this group? Servers will not be deleted.')) {
    emit('delete', groupId);
  }
}

function getGroupServerCount(groupId: string): number {
  // This would ideally come from a computed prop or store
  return props.groups.find(g => g.id === groupId)?.serverCount ?? 0;
}
</script>

<style scoped>
.groups-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}

.panel-header h4 {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  margin: 0;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.2s;
}

.btn-icon:hover:not(:disabled) {
  background: var(--bg-secondary);
  color: var(--fg);
}

.btn-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Group item */
.group-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: none;
  border: none;
  cursor: pointer;
  transition: background 0.15s;
  text-align: left;
  width: 100%;
}

.group-item:hover {
  background: var(--bg-secondary);
}

.group-item.active {
  background: rgba(255, 255, 255, 0.18);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
}

.group-item.active .group-name {
  color: var(--accent);
  font-weight: 500;
}

.group-item.hidden {
  opacity: 0.5;
}

.group-select {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
}

.group-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.group-name {
  flex: 1;
  font-size: 13px;
  color: var(--fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.group-count {
  font-size: 11px;
  color: var(--fg-muted);
  background: var(--bg-tertiary);
  padding: 2px 6px;
  border-radius: 10px;
  flex-shrink: 0;
}

.group-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.group-item:hover .group-actions {
  opacity: 1;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: var(--fg-muted);
  transition: all 0.15s;
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--fg);
}

.action-btn.danger:hover {
  background: var(--error-bg);
  color: var(--error);
}

/* Create/Edit form */
.group-create-form,
.group-edit-form {
  padding: 12px 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.group-create-form input,
.group-edit-form input {
  padding: 8px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  color: var(--fg);
}

.group-create-form input:focus,
.group-edit-form input:focus {
  outline: none;
  border-color: var(--accent);
}

.color-picker {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.color-option {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.color-option:hover {
  transform: scale(1.1);
}

.color-option.selected {
  border-color: var(--fg);
  box-shadow: 0 0 0 2px var(--bg-primary);
}

.form-actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

/* Group list */
.group-list {
  flex: 1;
  overflow-y: auto;
}

.group-item-wrapper {
  border-bottom: 1px solid var(--border);
}

/* Empty state */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 24px 16px;
  text-align: center;
}

.empty-state p {
  font-size: 13px;
  color: var(--fg-muted);
  margin: 0;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  transition: all 0.15s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-sm {
  padding: 5px 10px;
  font-size: 11px;
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover, var(--accent));
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
}

.btn-secondary:hover:not(:disabled) {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}
</style>
