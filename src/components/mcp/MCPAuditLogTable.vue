<template>
  <div class="audit-log-table">
    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="filter-group">
        <select v-model="localFilters.action" @change="handleFilterChange" class="filter-select">
          <option value="">All Actions</option>
          <option value="create">Create</option>
          <option value="update">Update</option>
          <option value="delete">Delete</option>
          <option value="health_check">Health Check</option>
          <option value="invoke">Invoke</option>
        </select>

        <input
          type="text"
          v-model="localFilters.serviceName"
          placeholder="Filter by service..."
          @input="debouncedFilterChange"
          class="filter-input"
        />

        <select v-model="localFilters.status" @change="handleFilterChange" class="filter-select">
          <option value="">All Status</option>
          <option value="success">Success</option>
          <option value="failure">Failure</option>
        </select>
      </div>

      <div class="filter-group">
        <input
          type="date"
          v-model="localFilters.dateFrom"
          @change="handleFilterChange"
          class="filter-input filter-date"
          placeholder="From"
        />

        <input
          type="date"
          v-model="localFilters.dateTo"
          @change="handleFilterChange"
          class="filter-input filter-date"
          placeholder="To"
        />

        <button class="btn btn-sm btn-secondary" @click="$emit('export', localFilters)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Export CSV
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>Loading audit log...</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="entries.length === 0" class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <polyline points="10 9 9 9 8 9"/>
      </svg>
      <p>No audit entries found</p>
      <span>Try adjusting your filters</span>
    </div>

    <!-- Table -->
    <div v-else class="table-container">
      <table class="audit-table">
        <thead>
          <tr>
            <th class="col-timestamp sortable" @click="handleSort('timestamp')">
              Timestamp
              <span v-if="sortColumn === 'timestamp'" class="sort-indicator">
                {{ sortDirection === 'asc' ? '↑' : '↓' }}
              </span>
            </th>
            <th class="col-actor sortable" @click="handleSort('actor')">
              Actor
              <span v-if="sortColumn === 'actor'" class="sort-indicator">
                {{ sortDirection === 'asc' ? '↑' : '↓' }}
              </span>
            </th>
            <th class="col-action sortable" @click="handleSort('action')">
              Action
              <span v-if="sortColumn === 'action'" class="sort-indicator">
                {{ sortDirection === 'asc' ? '↑' : '↓' }}
              </span>
            </th>
            <th class="col-service sortable" @click="handleSort('service')">
              Service
              <span v-if="sortColumn === 'service'" class="sort-indicator">
                {{ sortDirection === 'asc' ? '↑' : '↓' }}
              </span>
            </th>
            <th class="col-details">Details</th>
            <th class="col-status">Status</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="entry in entries" :key="entry.id" class="audit-row">
            <td class="col-timestamp">
              <span class="timestamp">{{ formatTimestamp(entry.createdAt) }}</span>
            </td>
            <td class="col-actor">
              <span class="actor">{{ entry.actor || 'System' }}</span>
            </td>
            <td class="col-action">
              <span class="action-badge" :class="entry.action">
                {{ formatAction(entry.action) }}
              </span>
            </td>
            <td class="col-service">
              <span class="service-name">{{ entry.serviceName || '—' }}</span>
            </td>
            <td class="col-details">
              <span class="details" :title="entry.details">
                {{ truncateDetails(entry.details) }}
              </span>
            </td>
            <td class="col-status">
              <span class="status-badge" :class="entry.status">
                {{ entry.status === 'success' ? 'Success' : 'Failed' }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="pagination">
      <div class="pagination-info">
        Showing {{ startItem }}-{{ endItem }} of {{ total }} entries
      </div>

      <div class="pagination-controls">
        <button
          class="page-btn"
          :disabled="page <= 1"
          @click="goToPage(page - 1)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
          Prev
        </button>

        <div class="page-numbers">
          <button
            v-for="p in visiblePages"
            :key="p"
            class="page-number"
            :class="{ active: p === page, ellipsis: p === '...' }"
            :disabled="p === '...'"
            @click="typeof p === 'number' && goToPage(p)"
          >
            {{ p }}
          </button>
        </div>

        <button
          class="page-btn"
          :disabled="page >= totalPages"
          @click="goToPage(page + 1)"
        >
          Next
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { debounce } from '@vueuse/core';
import type { MCPAuditEntry, MCPAuditFilters } from '@/types';

interface Props {
  entries: MCPAuditEntry[];
  total: number;
  page: number;
  pageSize?: number;
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  pageSize: 20,
  loading: false,
});

const emit = defineEmits<{
  (e: 'page-change', page: number): void;
  (e: 'filter-change', filters: MCPAuditFilters): void;
  (e: 'export', filters: MCPAuditFilters): void;
}>();

const localFilters = ref<MCPAuditFilters>({
  action: '',
  serviceName: '',
  status: '',
  dateFrom: '',
  dateTo: '',
});

const sortColumn = ref<string | null>(null);
const sortDirection = ref<'asc' | 'desc'>('desc');

const totalPages = computed(() => Math.ceil(props.total / props.pageSize));
const startItem = computed(() => (props.page - 1) * props.pageSize + 1);
const endItem = computed(() => Math.min(props.page * props.pageSize, props.total));

// Generate visible page numbers
const visiblePages = computed<(number | string)[]>(() => {
  const pages: (number | string)[] = [];
  const total = totalPages.value;
  const current = props.page;

  if (total <= 7) {
    for (let i = 1; i <= total; i++) {
      pages.push(i);
    }
  } else {
    pages.push(1);

    if (current > 3) {
      pages.push('...');
    }

    for (let i = Math.max(2, current - 1); i <= Math.min(total - 1, current + 1); i++) {
      if (!pages.includes(i)) {
        pages.push(i);
      }
    }

    if (current < total - 2) {
      pages.push('...');
    }

    if (!pages.includes(total)) {
      pages.push(total);
    }
  }

  return pages;
});

function formatTimestamp(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}

function formatAction(action: string): string {
  const labels: Record<string, string> = {
    create: 'Create',
    update: 'Update',
    delete: 'Delete',
    health_check: 'Health',
    invoke: 'Invoke',
  };
  return labels[action] || action;
}

function truncateDetails(details: string): string {
  if (!details) return '—';
  try {
    const parsed = JSON.parse(details);
    const summary = parsed.toolName || parsed.name || parsed;
    return typeof summary === 'string' && summary.length > 40
      ? summary.substring(0, 37) + '...'
      : String(summary);
  } catch {
    return details.length > 40 ? details.substring(0, 37) + '...' : details;
  }
}

function handleFilterChange() {
  emit('filter-change', localFilters.value);
}

const debouncedFilterChange = debounce(handleFilterChange, 300);

function handleSort(column: string) {
  if (sortColumn.value === column) {
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
  } else {
    sortColumn.value = column;
    sortDirection.value = 'desc';
  }
}

function goToPage(targetPage: number) {
  if (targetPage < 1 || targetPage > totalPages.value) return;
  emit('page-change', targetPage);
}

// Reset filters when component mounts
watch(
  () => props.entries,
  () => {
    // Could reset local state here if needed
  }
);
</script>

<style scoped>
.audit-log-table {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Filter Bar */
.filter-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-select,
.filter-input {
  padding: 6px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 12px;
  color: var(--fg);
  transition: border-color 0.2s;
}

.filter-select:focus,
.filter-input:focus {
  outline: none;
  border-color: var(--accent);
}

.filter-date {
  width: 130px;
}

/* Loading & Empty States */
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px 24px;
  text-align: center;
}

.loading-state {
  color: var(--fg-muted);
}

.empty-state svg {
  color: var(--fg-muted);
  opacity: 0.5;
}

.empty-state p {
  font-size: 14px;
  font-weight: 500;
  color: var(--fg);
  margin: 0;
}

.empty-state span {
  font-size: 12px;
  color: var(--fg-muted);
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Table */
.table-container {
  overflow-x: auto;
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.audit-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.audit-table th,
.audit-table td {
  padding: 10px 14px;
  text-align: left;
}

.audit-table th {
  background: var(--bg-secondary);
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--fg-muted);
  white-space: nowrap;
}

.audit-table th.sortable {
  cursor: pointer;
  user-select: none;
}

.audit-table th.sortable:hover {
  color: var(--fg);
}

.sort-indicator {
  margin-left: 4px;
  color: var(--accent);
}

.audit-row {
  border-top: 1px solid rgba(255, 255, 255, 0.15);
  transition: background 0.15s;
}

.audit-row:hover {
  background: rgba(255, 255, 255, 0.08);
}

.col-timestamp { width: 160px; }
.col-actor { width: 100px; }
.col-action { width: 100px; }
.col-service { width: 180px; }
.col-details { min-width: 150px; }
.col-status { width: 80px; }

.timestamp {
  font-family: 'JetBrains Mono', 'SF Mono', Monaco, Consolas, monospace;
  font-size: 11px;
  color: var(--fg-muted);
}

.actor {
  color: var(--fg);
}

.action-badge {
  display: inline-block;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.action-badge.create {
  background: rgba(16, 185, 129, 0.15);
  color: #10B981;
}

.action-badge.update {
  background: rgba(59, 130, 246, 0.15);
  color: #3B82F6;
}

.action-badge.delete {
  background: rgba(239, 68, 68, 0.15);
  color: #EF4444;
}

.action-badge.health_check {
  background: rgba(245, 158, 11, 0.15);
  color: #F59E0B;
}

.action-badge.invoke {
  background: rgba(139, 92, 246, 0.15);
  color: #8B5CF6;
}

.service-name {
  color: var(--fg);
  font-weight: 500;
}

.details {
  color: var(--fg-muted);
  font-size: 12px;
  cursor: default;
}

.status-badge {
  display: inline-block;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.status-badge.success {
  background: rgba(16, 185, 129, 0.15);
  color: #10B981;
}

.status-badge.failure {
  background: rgba(239, 68, 68, 0.15);
  color: #EF4444;
}

/* Pagination */
.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-card);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  border: 1px solid rgba(255, 255, 255, 0.35);
  border-radius: var(--radius);
  box-shadow: var(--shadow-md);
}

.pagination-info {
  font-size: 12px;
  color: var(--fg-muted);
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 12px;
  color: var(--fg);
  cursor: pointer;
  transition: all 0.15s;
}

.page-btn:hover:not(:disabled) {
  background: var(--bg-tertiary);
  border-color: var(--accent);
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-numbers {
  display: flex;
  gap: 4px;
}

.page-number {
  min-width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 12px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.page-number:hover:not(:disabled):not(.ellipsis) {
  background: var(--bg-secondary);
  color: var(--fg);
}

.page-number.active {
  background: var(--accent);
  color: white;
  font-weight: 500;
}

.page-number.ellipsis {
  cursor: default;
  color: var(--fg-muted);
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-sm {
  padding: 6px 10px;
  font-size: 11px;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--fg-muted);
}

.btn-secondary:hover {
  border-color: rgba(255, 255, 255, 0.30);
  color: var(--accent);
  background: rgba(255, 255, 255, 0.25);
}
</style>
