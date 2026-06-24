/* Forge — AI Env Manager · Application Logic */

// === Debounce utility ===
function debounce(fn, ms) {
  ms = ms || DEBOUNCE_MS;
  let timer;
  return function(...args) {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), ms);
  };
}

// === Toast notification ===
function showNotification(msg, type) {
  const existing = document.querySelector('.toast');
  if (existing) existing.remove();
  const t = document.createElement('div');
  t.className = 'toast ' + (type || '');
  const icons = {
    success:'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2.5" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg>',
    error:'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--error)" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>',
    info:'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--info)" stroke-width="2.5" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>'
  };
  t.innerHTML = (icons[type] || icons.info) + '<span>' + msg + '</span>';
  document.body.appendChild(t);
  setTimeout(() => {
    t.style.animation = 'toastOut var(--t-slow) forwards';
    setTimeout(() => t.remove(), TOAST_FADE);
  }, TOAST_DURATION);
}

// === Confirm dialog ===
let confirmResolve = null;
function showConfirm(title, message, actionText) {
  document.getElementById('confirmTitle').textContent = title;
  document.getElementById('confirmMessage').textContent = message;
  document.getElementById('confirmAction').textContent = actionText || 'Confirm';
  document.getElementById('confirmDialog').style.display = 'flex';
  return new Promise(r => { confirmResolve = r });
}
function closeConfirm(result) {
  document.getElementById('confirmDialog').style.display = 'none';
  if (confirmResolve) { confirmResolve(result); confirmResolve = null }
}

// === Modal open/close ===
function openModal(id) { const m = document.getElementById(id); if (m) m.style.display = 'flex' }
function closeModal(id) { const m = document.getElementById(id); if (m) m.style.display = 'none' }

// === Plugin detail modal ===
function openPluginDetail(p) {
  document.getElementById('pluginDetailTitle').textContent = p.name;
  document.getElementById('pluginDetailDesc').textContent = p.desc;
  document.getElementById('pluginDetailVersion').textContent = 'v' + p.version;
  document.getElementById('pluginDetailAuthor').textContent = p.author || '—';
  document.getElementById('pluginDetailSoftware').textContent = p.software || '—';
  document.getElementById('pluginDetailDate').textContent = p.installedAt || '—';
  openModal('pluginDetailModal');
}

// === Sidebar toggle ===
function toggleSidebar() {
  const sidebar = document.querySelector('.sidebar');
  if (!sidebar) return;
  sidebar.classList.toggle('collapsed');
  localStorage.setItem('sidebarCollapsed', sidebar.classList.contains('collapsed'));
}

// === Settings toggle ===
function toggleSetting(el, key) {
  el.classList.toggle('on');
  localStorage.setItem(key, el.classList.contains('on'));
}

// === Mobile tab bar ===
function updateMobileTab(el) {
  document.querySelectorAll('.mobile-tab-item').forEach(t => t.classList.remove('active'));
  if (el) el.classList.add('active');
}
function toggleMobileMore(el) {
  const menu = document.getElementById('mobileMoreMenu');
  if (!menu) return;
  const isOpen = menu.style.display !== 'none';
  menu.style.display = isOpen ? 'none' : 'block';
  if (el) el.classList.toggle('active', !isOpen);
}
function closeMobileMore() {
  const menu = document.getElementById('mobileMoreMenu');
  if (menu) menu.style.display = 'none';
}

// === CLI Tools operations ===
function startInstall(key) {
  operations.set(key, {stage:'preparing',progress:0,message:'…'});
  renderCurrentView();
  simulateProgress(key, 'installing');
}
function startUpgrade(key) {
  operations.set(key, {stage:'preparing',progress:0,message:'…'});
  renderCurrentView();
  simulateProgress(key, 'downloading');
}
function cancelOperation(key) {
  operations.set(key, {stage:'cancelled',progress:0,message:'⊘'});
  renderCurrentView();
  showNotification('Operation cancelled', 'info');
  setTimeout(() => { operations.delete(key); renderCurrentView() }, CANCEL_CLEANUP);
}
function simulateProgress(key, activeStage) {
  let progress = 0;
  const interval = setInterval(() => {
    const op = operations.get(key);
    if (!op || op.stage === 'cancelled') { clearInterval(interval); return }
    progress += Math.random() * 15 + 5;
    if (progress >= 100) {
      clearInterval(interval);
      operations.set(key, {stage:'completed',progress:100,message:'✔'});
      renderCurrentView();
      showNotification(key + ' operation completed', 'success');
      setTimeout(() => { operations.delete(key); renderCurrentView() }, STAGE_TIMEOUT);
      return;
    }
    const stage = progress < 30 ? 'preparing' : progress < 60 ? activeStage : 'verifying';
    operations.set(key, {stage,progress:Math.round(progress),message:STAGE_CONFIG[stage].icon});
    renderCurrentView();
  }, PROGRESS_INTERVAL);
}

// === Plugin sync ===
function syncPluginToTool(pluginName, toolKey, chipEl) {
  const plugin = plugins.find(p => p.name === pluginName);
  const tool = cliTools.find(t => t.key === toolKey);
  if (!plugin || !tool) return;
  const statusEl = chipEl.querySelector('.chip-status');
  const spinIcon = '<svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>';
  const checkIcon = '<svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg>';
  const isSynced = plugin.syncedWith && plugin.syncedWith.includes(toolKey);
  if (isSynced) {
    chipEl.classList.remove('synced'); chipEl.classList.add('syncing');
    statusEl.innerHTML = spinIcon;
    setTimeout(() => {
      chipEl.classList.remove('syncing'); chipEl.classList.add('synced');
      statusEl.innerHTML = checkIcon;
      showNotification(pluginName + ' re-synced with ' + tool.name, 'success');
    }, SYNC_RE_DURATION);
  } else {
    chipEl.classList.remove('unsynced'); chipEl.classList.add('syncing');
    statusEl.innerHTML = spinIcon;
    setTimeout(() => {
      if (!plugin.syncedWith) plugin.syncedWith = [];
      plugin.syncedWith.push(toolKey);
      chipEl.classList.remove('syncing'); chipEl.classList.add('synced');
      statusEl.innerHTML = checkIcon;
      const card = chipEl.closest('.card');
      const nameDiv = card.querySelector('div[style*="font-weight:600"]');
      let countBadge = nameDiv.querySelector('.sync-count');
      if (countBadge) { countBadge.textContent = plugin.syncedWith.length + ' synced' }
      else { nameDiv.insertAdjacentHTML('beforeend', '<span class="sync-count">' + plugin.syncedWith.length + ' synced</span>') };
      showNotification(pluginName + ' synced to ' + tool.name, 'success');
    }, SYNC_NEW_DURATION);
  }
}

// === Skill tool toggle ===
function toggleSkillTool(skillName, toolKey, iconEl) {
  const skill = skills.find(s => s.name === skillName);
  const tool = cliTools.find(t => t.key === toolKey);
  if (!skill || !tool) return;
  if (!skill.syncedWith) skill.syncedWith = [];
  const isSynced = skill.syncedWith.includes(toolKey);
  iconEl.style.transform = 'scale(0.8)';
  setTimeout(() => {
    if (isSynced) {
      skill.syncedWith = skill.syncedWith.filter(k => k !== toolKey);
      iconEl.classList.remove('synced'); iconEl.classList.add('unsynced');
      iconEl.style.background = 'rgba(154,154,154,0.08)'; iconEl.style.color = 'var(--fg-ghost)';
      iconEl.style.borderColor = 'var(--border)';
      iconEl.title = tool.name + ' — click to sync';
      showNotification(skillName + ' unsynced from ' + tool.name, 'info');
    } else {
      skill.syncedWith.push(toolKey);
      iconEl.classList.remove('unsynced'); iconEl.classList.add('synced');
      iconEl.style.background = tool.color + '22'; iconEl.style.color = tool.color;
      iconEl.style.borderColor = tool.color;
      iconEl.title = tool.name + ' — synced';
      showNotification(skillName + ' synced to ' + tool.name, 'success');
    }
    iconEl.style.transform = 'scale(1)';
  }, 150);
}

// === Marketplace install ===
function installMarketplacePlugin(name) {
  const p = marketplacePlugins.find(x => x.name === name);
  if (!p) return;
  p.installed = true;
  showNotification(name + ' installed successfully', 'success');
  renderMarketplace();
  const installedCount = marketplacePlugins.filter(x => x.installed).length;
  const countEl = document.getElementById('pluginsCount');
  if (countEl) countEl.textContent = installedCount + ' installed';
}

// === Marketplace filter ===
let marketplaceSourceFilter = 'all';
let marketplaceDebounceTimer = null;
function filterMarketplace() {
  clearTimeout(marketplaceDebounceTimer);
  marketplaceDebounceTimer = setTimeout(() => {
    const q = (document.getElementById('marketplaceSearch')?.value || '').toLowerCase().trim();
    const cat = document.getElementById('marketplaceCategoryFilter')?.value || 'all';
    const sort = document.getElementById('marketplaceSortFilter')?.value || 'popular';
    let filtered = marketplacePlugins.slice();
    if (marketplaceSourceFilter !== 'all') filtered = filtered.filter(p => p.source === marketplaceSourceFilter);
    if (cat !== 'all') filtered = filtered.filter(p => p.categories.includes(cat));
    if (q) filtered = filtered.filter(p => p.name.toLowerCase().includes(q) || p.desc.toLowerCase().includes(q) || p.author.toLowerCase().includes(q));
    if (sort === 'popular') filtered.sort((a, b) => b.downloads - a.downloads);
    else if (sort === 'newest') filtered.sort((a, b) => b.version.localeCompare(a.version));
    else if (sort === 'name') filtered.sort((a, b) => a.name.localeCompare(b.name));
    renderMarketplace(filtered);
    const allTabs = document.querySelectorAll('#marketplaceSourceTabs .tab-count');
    if (allTabs.length >= 4) {
      allTabs[0].textContent = filtered.length;
      const srcCounts = {};
      filtered.forEach(p => { srcCounts[p.source] = (srcCounts[p.source] || 0) + 1 });
      allTabs[1].textContent = srcCounts['forge-official'] || 0;
      allTabs[2].textContent = srcCounts['community-hub'] || 0;
      allTabs[3].textContent = srcCounts['ai-tools-pack'] || 0;
    }
  }, DEBOUNCE_MS);
}

// === Tab switching ===
function switchPluginTab(tab, el) {
  document.querySelectorAll('#pluginsTabs .tab-item').forEach(t => t.classList.remove('active'));
  el.classList.add('active');
  document.getElementById('plugins-installed').style.display = tab === 'installed' ? '' : 'none';
  document.getElementById('plugins-marketplace').style.display = tab === 'marketplace' ? '' : 'none';
  document.getElementById('plugins-sources').style.display = tab === 'sources' ? '' : 'none';
  if (tab === 'marketplace') renderMarketplace();
}
function switchMarketplaceSource(source, el) {
  document.querySelectorAll('#marketplaceSourceTabs .source-tab').forEach(t => t.classList.remove('active'));
  el.classList.add('active');
  marketplaceSourceFilter = source;
  renderMarketplace();
}
function switchMCPTab(tab, el) {
  document.querySelectorAll('#mcpTabs .tab-item').forEach(t => t.classList.remove('active'));
  el.classList.add('active');
  document.getElementById('mcp-services').style.display = tab === 'services' ? '' : 'none';
  document.getElementById('mcp-groups').style.display = tab === 'groups' ? '' : 'none';
  document.getElementById('mcp-audit').style.display = tab === 'audit' ? '' : 'none';
}
function switchSkillsSource(source, el) {
  document.querySelectorAll('#skillsSourceTabs .source-tab').forEach(t => t.classList.remove('active'));
  el.classList.add('active');
  filterSkills();
}

// === Filter functions ===
const debouncedFilterSoftware = debounce(filterSoftware);
function filterSoftwareTab(tab, el) {
  if (el) { el.parentElement.querySelectorAll('.tab-item').forEach(t => t.classList.remove('active')); el.classList.add('active') }
  const statusFilter = document.getElementById('softwareStatusFilter');
  if (statusFilter) {
    if (tab === 'detected') statusFilter.value = 'detected';
    else if (tab === 'not-found') statusFilter.value = 'not-found';
    else statusFilter.value = 'all';
  }
  filterSoftware();
}
function filterSoftware() {
  const search = (document.getElementById('softwareSearch')?.value || '').toLowerCase();
  const tier = document.getElementById('softwareTierFilter')?.value || 'all';
  const platform = document.getElementById('softwarePlatformFilter')?.value || 'all';
  const status = document.getElementById('softwareStatusFilter')?.value || 'all';
  const platformMap = {macos:['Cursor','Claude Desktop','Windsurf','Cody','Copilot'],windows:['Copilot','Continue','Windsurf'],linux:['Continue','Windsurf'],cross:['Cursor','Claude Desktop','Windsurf','Continue','Cody','Copilot']};
  const filtered = software.filter(sw => {
    if (search && !sw.name.toLowerCase().includes(search) && !sw.configPath.toLowerCase().includes(search)) return false;
    if (tier !== 'all' && sw.tier !== tier) return false;
    if (status === 'detected' && !sw.installed) return false;
    if (status === 'not-found' && sw.installed) return false;
    if (platform !== 'auto' && platform !== 'all' && platformMap[platform] && !platformMap[platform].includes(sw.name)) return false;
    return true;
  });
  document.getElementById('softwareList').innerHTML = filtered.length ? filtered.map(renderSoftware).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg><h3>No software found</h3><p>Try adjusting your search or filter criteria.</p></div>';
  document.getElementById('softwareCount').textContent = filtered.length + ' apps';
}

const debouncedFilterPlugins = debounce(filterPlugins);
function filterPlugins() {
  const search = (document.getElementById('pluginSearch')?.value || '').toLowerCase();
  const status = document.getElementById('pluginStatusFilter')?.value || 'all';
  const filtered = plugins.filter(p => {
    if (search && !p.name.toLowerCase().includes(search) && !p.desc.toLowerCase().includes(search)) return false;
    if (status === 'enabled' && !p.enabled) return false;
    if (status === 'disabled' && p.enabled) return false;
    return true;
  });
  document.getElementById('pluginsList').innerHTML = filtered.length ? filtered.map(renderPlugin).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg><h3>No plugins found</h3><p>Try adjusting your search or filter criteria.</p></div>';
  document.getElementById('pluginsCount').textContent = filtered.length + ' installed';
}

const debouncedFilterSkills = debounce(filterSkills);
function filterSkills() {
  const search = (document.getElementById('skillsSearch')?.value || '').toLowerCase();
  const type = document.getElementById('skillsTypeFilter')?.value || 'all';
  const status = document.getElementById('skillsStatusFilter')?.value || 'all';
  const source = document.querySelector('#skillsSourceTabs .source-tab.active')?.textContent.split(' ')[0].toLowerCase() || 'all';
  const filtered = skills.filter(s => {
    if (search && !s.name.toLowerCase().includes(search) && !s.desc.toLowerCase().includes(search)) return false;
    if (type !== 'all' && s.type !== type) return false;
    if (status === 'enabled' && !s.enabled) return false;
    if (status === 'disabled' && s.enabled) return false;
    if (source !== 'all' && source !== 'skills') {
      if (s.source !== source) return false;
    }
    return true;
  });
  document.getElementById('skillsList').innerHTML = filtered.length ? filtered.map(renderSkill).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg><h3>No skills found</h3><p>Try adjusting your search or filter criteria.</p></div>';
  document.getElementById('skillsCount').textContent = filtered.length + ' skills';
}

const debouncedFilterAgents = debounce(filterAgents);
function filterAgents() {
  const search = (document.getElementById('agentsSearch')?.value || '').toLowerCase();
  const dept = document.getElementById('agentDeptFilter')?.value || 'all';
  const source = document.getElementById('agentSourceFilter')?.value || 'all';
  const filtered = agents.filter(a => {
    if (search && !a.name.toLowerCase().includes(search) && !a.desc.toLowerCase().includes(search)) return false;
    if (dept !== 'all' && a.department !== dept) return false;
    if (source !== 'all' && a.source !== source) return false;
    return true;
  });
  document.getElementById('agentsList').innerHTML = filtered.length ? filtered.map(renderAgent).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg><h3>No agents found</h3><p>Try adjusting your search or filter criteria.</p></div>';
  document.getElementById('agentsCount').textContent = filtered.length + ' agents';
}

const debouncedFilterMCP = debounce(filterMCP);
function filterMCP() {
  const search = (document.getElementById('mcpSearch')?.value || '').toLowerCase();
  const health = document.getElementById('mcpHealthFilter')?.value || 'all';
  const auth = document.getElementById('mcpAuthFilter')?.value || 'all';
  const filtered = mcpServers.filter(s => {
    if (search && !s.name.toLowerCase().includes(search) && !s.endpoint.toLowerCase().includes(search)) return false;
    if (health === 'healthy' && !s.healthy) return false;
    if (health === 'unhealthy' && s.healthy) return false;
    if (auth !== 'all' && s.auth !== auth) return false;
    return true;
  });
  document.getElementById('mcpList').innerHTML = filtered.length ? filtered.map(renderMCPServer).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4"/></svg><h3>No MCP servers found</h3><p>Try adjusting your search or filter criteria.</p></div>';
  document.getElementById('mcpCount').textContent = filtered.length + ' servers';
}

const debouncedFilterRules = debounce(filterRules);
function filterRules() {
  const search = (document.getElementById('rulesSearch')?.value || '').toLowerCase();
  const type = document.getElementById('rulesTypeFilter')?.value || 'all';
  const status = document.getElementById('rulesStatusFilter')?.value || 'all';
  const sw = document.getElementById('rulesSoftwareFilter')?.value || 'all';
  const filtered = rulesFiles.filter(r => {
    if (search && !r.name.toLowerCase().includes(search)) return false;
    if (type !== 'all' && r.type !== type) return false;
    if (status === 'active' && !r.active) return false;
    if (status === 'inactive' && r.active) return false;
    if (sw !== 'all' && r.software !== sw) return false;
    return true;
  });
  document.getElementById('rulesList').innerHTML = filtered.length ? filtered.map(renderRule).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg><h3>No rules found</h3><p>Try adjusting your search or filter criteria.</p></div>';
  document.getElementById('rulesCount').textContent = filtered.length + ' files';
}

const debouncedFilterBackups = debounce(filterBackups);
function filterBackups() {
  const search = (document.getElementById('backupSearch')?.value || '').toLowerCase();
  const type = document.getElementById('backupTypeFilter')?.value || 'all';
  const filtered = backups.filter(b => {
    if (search && !b.name.toLowerCase().includes(search) && !b.date.includes(search) && !b.includes.some(t => t.toLowerCase().includes(search))) return false;
    if (type !== 'all' && b.type !== type) return false;
    return true;
  });
  document.getElementById('backupList').innerHTML = filtered.length ? filtered.map(renderBackup).join('') : '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--fg-ghost)" stroke-width="1.5" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg><h3>No backups found</h3><p>Try adjusting your search or filter criteria.</p></div>';
}

const debouncedFilterSources = debounce(filterSources);
function filterSources(q) {
  q = (q || '').toLowerCase();
  const filtered = sourcesData.filter(s => s.name.toLowerCase().includes(q) || s.url.toLowerCase().includes(q));
  renderSources(filtered);
}
function filterSourcesByStatus(v) {
  if (v === 'all') return renderSources();
  const filtered = sourcesData.filter(s => v === 'installed' ? s.installed : !s.installed);
  renderSources(filtered);
}

// === Backup restore ===
async function restoreBackup(name, date) {
  const confirmed = await showConfirm('Restore Backup', 'Are you sure you want to restore from ' + name + ' (' + date + ')? This will overwrite current settings.', 'Restore');
  if (confirmed) {
    showNotification('Restoring from ' + date + '…', 'info');
    setTimeout(() => {
      showNotification('Restore completed successfully', 'success');
    }, 2000);
  }
}

// === Theme system ===
function renderThemeGrid() {
  const grid = document.getElementById('themeGrid');
  if (!grid) return;
  grid.innerHTML = THEMES.map(t =>
    '<div class="theme-card'+(t.active?' active':'')+'" data-od-id="theme-card-'+t.id+'" data-theme="'+t.id+'" title="'+t.desc+'" onclick="selectTheme(\''+t.id+'\')"><div class="theme-preview">'+t.colors.map(c => '<span style="background:'+c+'"></span>').join('')+'</div><div class="theme-info"><div class="name">'+t.name+'</div></div></div>'
  ).join('');
}

function selectTheme(id) {
  THEMES.forEach(t => t.active = (t.id === id));
  document.querySelectorAll('.theme-card').forEach(c => {
    c.classList.toggle('active', c.dataset.theme === id);
  });
  const theme = THEMES.find(t => t.id === id);
  if (!theme) return;
  localStorage.setItem('selectedTheme', id);
  const [bg, surface, border, fg, accent, warn] = theme.colors;
  const fgR = parseInt(fg.slice(1,3),16), fgG = parseInt(fg.slice(3,5),16), fgB = parseInt(fg.slice(5,7),16);
  const bgR = parseInt(bg.slice(1,3),16), bgG = parseInt(bg.slice(3,5),16), bgB = parseInt(bg.slice(5,7),16);
  const fgLum = 0.299*fgR + 0.587*fgG + 0.114*fgB;
  const bgLum = 0.299*bgR + 0.587*bgG + 0.114*bgB;
  const isDark = fgLum > bgLum;
  requestAnimationFrame(() => {
    const r = document.documentElement.style;
    r.setProperty('--bg', bg);
    r.setProperty('--surface', surface);
    r.setProperty('--fg', fg);
    r.setProperty('--fg-title', isDark ? fg : '#111111');
    r.setProperty('--fg-muted', isDark ? 'rgba(255,255,255,0.6)' : '#5C5C5C');
    r.setProperty('--fg-ghost', isDark ? 'rgba(255,255,255,0.35)' : '#9A9A9A');
    r.setProperty('--accent', accent);
    r.setProperty('--accent-hover', accent + 'dd');
    r.setProperty('--accent-press', accent + 'bb');
    r.setProperty('--accent-glow', accent + '22');
    r.setProperty('--warn', warn);
    r.setProperty('--border', isDark ? 'rgba(255,255,255,0.10)' : 'rgba(255,255,255,0.20)');
    r.setProperty('--border-hover', isDark ? 'rgba(255,255,255,0.30)' : 'rgba(255,255,255,0.35)');
    r.setProperty('--border-window', isDark ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.18)');
    r.setProperty('--glass-bg', isDark ? 'rgba(255,255,255,0.06)' : 'rgba(255,255,255,0.45)');
    r.setProperty('--glass-bg-hover', isDark ? 'rgba(255,255,255,0.10)' : 'rgba(255,255,255,0.58)');
    r.setProperty('--glass-sidebar', isDark ? 'rgba(255,255,255,0.04)' : 'rgba(255,255,255,0.35)');
    r.setProperty('--glass-topbar', isDark ? 'rgba(255,255,255,0.04)' : 'rgba(255,255,255,0.38)');
    r.setProperty('--glass-window', isDark ? 'rgba(255,255,255,0.03)' : 'rgba(255,255,255,0.25)');
    r.setProperty('--glass-input', isDark ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.40)');
    r.setProperty('--shadow', isDark ? '0 2px 16px rgba(0,0,0,0.25)' : '0 2px 16px rgba(0,0,0,0.04)');
    r.setProperty('--shadow-hover', isDark ? '0 6px 24px rgba(0,0,0,0.35)' : '0 6px 24px rgba(0,0,0,0.07)');
    r.setProperty('--shadow-btn', isDark ? '0 1px 6px rgba(0,0,0,0.30)' : '0 1px 6px rgba(0,0,0,0.08)');
    r.setProperty('--shadow-inner', isDark ? 'inset 0 1px 2px rgba(0,0,0,0.20)' : 'inset 0 1px 2px rgba(0,0,0,0.04)');
    r.setProperty('--tint-warm', isDark ? 'rgba(195,178,155,0.06)' : 'rgba(195,178,155,0.10)');
    r.setProperty('--tint-cool', isDark ? 'rgba(195,178,155,0.05)' : 'rgba(195,178,155,0.08)');
    const tintWarm = isDark ? 'rgba('+Math.min(bgR+40,255)+','+Math.min(bgG+35,255)+','+Math.min(bgB+25,255)+',0.12)' : 'rgba(195,178,155,0.15)';
    const tintCool = isDark ? 'rgba('+Math.max(bgR-10,0)+','+Math.max(bgG-5,0)+','+Math.min(bgB+20,255)+',0.10)' : 'rgba(160,170,190,0.18)';
    const tintMid = isDark ? 'rgba('+Math.min(bgR+20,255)+','+Math.min(bgG+18,255)+','+Math.min(bgB+12,255)+',0.08)' : 'rgba(200,195,185,0.12)';
    const gridFine = isDark ? 'rgba(255,255,255,0.02)' : 'rgba(0,0,0,0.025)';
    const gridCoarse = isDark ? 'rgba(255,255,255,0.01)' : 'rgba(0,0,0,0.015)';
    document.body.style.backgroundImage =
      'linear-gradient(160deg,'+bg+' 0%,'+surface+' 25%,'+bg+' 50%,'+surface+' 75%,'+bg+' 100%),'
      +'radial-gradient(ellipse 120% 80% at 0% 0%,'+tintWarm+' 0%,transparent 50%),'
      +'radial-gradient(ellipse 100% 80% at 100% 100%,'+tintCool+' 0%,transparent 50%),'
      +'radial-gradient(ellipse 80% 60% at 60% 30%,'+tintMid+' 0%,transparent 45%),'
      +'linear-gradient('+gridFine+' 1px,transparent 1px),'
      +'linear-gradient(90deg,'+gridFine+' 1px,transparent 1px),'
      +'linear-gradient('+gridCoarse+' 1px,transparent 1px),'
      +'linear-gradient(90deg,'+gridCoarse+' 1px,transparent 1px)';
    document.body.style.backgroundSize = '100% 100%,100% 100%,100% 100%,100% 100%,24px 24px,24px 24px,96px 96px,96px 96px';
    let styleTag = document.getElementById('theme-dynamic');
    if (!styleTag) { styleTag = document.createElement('style'); styleTag.id = 'theme-dynamic'; document.head.appendChild(styleTag) }
    const beforeWarm = isDark ? 'rgba('+Math.min(bgR+35,255)+','+Math.min(bgG+30,255)+','+Math.min(bgB+20,255)+',0.12)' : 'rgba(185,170,150,0.15)';
    const beforeCool = isDark ? 'rgba('+Math.max(bgR-15,0)+','+Math.max(bgG-10,0)+','+Math.min(bgB+25,255)+',0.10)' : 'rgba(155,165,185,0.12)';
    const beforeMid = isDark ? 'rgba('+Math.min(bgR+25,255)+','+Math.min(bgG+22,255)+','+Math.min(bgB+15,255)+',0.06)' : 'rgba(200,195,185,0.08)';
    styleTag.textContent = 'body::before{background:radial-gradient(ellipse 70% 50% at 10% 15%,'+beforeWarm+' 0%,transparent 50%),radial-gradient(ellipse 60% 50% at 90% 80%,'+beforeCool+' 0%,transparent 50%),radial-gradient(ellipse 50% 40% at 50% 50%,'+beforeMid+' 0%,transparent 45%)}body::after{opacity:'+(isDark?0.15:0.30)+'}';
  });
  showNotification('Theme changed to ' + theme.name, 'success');
}

// === Unified click delegation ===
document.addEventListener('click', e => {
  const tChip = e.target.closest('.target-chip');
  if (tChip && tChip.closest('.target-grid')) {
    tChip.classList.toggle('selected');
    const key = tChip.dataset.key;
    const tool = agentTargetTools.find(t => t.key === key);
    const dot = tChip.querySelector('.chip-dot');
    if (dot && tool) dot.style.background = tChip.classList.contains('selected') ? tool.color : 'rgba(154,154,154,0.3)';
    const section = tChip.closest('.agent-targets-section');
    if (section) {
      const selected = tChip.closest('.target-grid').querySelectorAll('.target-chip.selected').length;
      const label = section.querySelector('.agent-targets-label');
      if (label) label.textContent = 'Install to · ' + selected + ' selected';
      const card = section.closest('.agent-card');
      if (card) { const btn = card.querySelector('.btn-primary'); if (btn) btn.textContent = 'Install (' + selected + ')' }
    }
    return;
  }
  const gChip = e.target.closest('.group-chip');
  if (gChip && gChip.closest('.group-chips')) {
    gChip.closest('.group-chips').querySelectorAll('.group-chip').forEach(c => c.classList.remove('active'));
    gChip.classList.add('active');
    return;
  }
  const dTab = e.target.closest('.detail-tab');
  if (dTab) {
    dTab.parentElement.querySelectorAll('.detail-tab').forEach(t => t.classList.remove('active'));
    dTab.classList.add('active');
    const tabName = dTab.textContent.trim().toLowerCase();
    const modal = dTab.closest('.modal');
    if (modal) {
      modal.querySelectorAll('.tab-content').forEach(c => c.style.display = 'none');
      const tabMap = {overview:'pluginTabOverview',skills:'pluginTabSkills',commands:'pluginTabCommands',hooks:'pluginTabHooks',mcp:'pluginTabMCP',lsp:'pluginTabLSP'};
      const targetId = tabMap[tabName];
      if (targetId) document.getElementById(targetId).style.display = '';
    }
    return;
  }
  const sTab = e.target.closest('.source-tab');
  if (sTab && !sTab.closest('#skillsSourceTabs')) {
    sTab.parentElement.querySelectorAll('.source-tab').forEach(t => t.classList.remove('active'));
    sTab.classList.add('active');
  }
});

// === Plugin card click-to-detail ===
document.addEventListener('click', e => {
  const chip = e.target.closest('.cli-sync-chip');
  const btn = e.target.closest('.btn-group,.btn,.btn-icon,.toggle-wrap,.toggle');
  if (chip || btn) return;
  const card = e.target.closest('.card.plugin-card');
  if (card) {
    const name = card.dataset.name;
    const p = plugins.find(x => x.name === name);
    if (p) openPluginDetail(p);
  }
});

// === Escape key ===
document.addEventListener('keydown', e => {
  if (e.key === 'Escape') {
    document.querySelectorAll('.modal-overlay').forEach(m => m.style.display = 'none');
    document.getElementById('confirmDialog').style.display = 'none';
  }
});

// === Filter: CLI Tools ===
function filterCLITools(filter, el) {
  if (el) {
    el.parentElement.querySelectorAll('.tab-item').forEach(t => t.classList.remove('active'));
    el.classList.add('active');
  }
  const filtered = cliTools.filter(t => {
    if (filter === 'installed') return t.installed;
    if (filter === 'available') return !t.installed;
    if (filter === 'updates') return t.needsUpdate;
    return true;
  });
  document.getElementById('cliToolsListFull').innerHTML = filtered.map(renderCLITool).join('');
}

// === Generic source tab switching ===
document.addEventListener('click', e => {
  const sTab = e.target.closest('.source-tab');
  if (sTab && !sTab.closest('#skillsSourceTabs') && !sTab.closest('#marketplaceSourceTabs')) {
    sTab.parentElement.querySelectorAll('.source-tab').forEach(t => t.classList.remove('active'));
    sTab.classList.add('active');
  }
});
