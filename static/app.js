:root {
  --bg: #0b0b0f;
  --panel: #14141a;
  --panel-alt: #1d1d24;
  --border: #2b2b33;
  --text: #edf2f7;
  --muted: #a0aec0;
  --red: #ef4444;
  --red-2: #991b1b;
  --green: #22c55e;
  --orange: #f59e0b;
  --shadow: rgba(239, 68, 68, 0.2);
}

* { box-sizing: border-box; }
html, body {
  margin: 0;
  height: 100%;
  background: radial-gradient(circle at top, #1d080a 0%, var(--bg) 35%, #07070b 100%);
  color: var(--text);
  font-family: 'Segoe UI', Tahoma, sans-serif;
}
body { padding: 22px; }
button, input, textarea { font: inherit; }

.app-shell {
  max-width: 1500px;
  margin: 0 auto;
  min-height: calc(100vh - 44px);
  display: flex;
  gap: 22px;
}

.sidebar {
  width: 270px;
  background: rgba(20,20,26,.92);
  border: 1px solid var(--border);
  border-radius: 18px;
  padding: 18px;
  box-shadow: 0 20px 60px rgba(0,0,0,.35);
}

.brand {
  font-size: 28px;
  font-weight: 900;
  letter-spacing: .06em;
  margin-bottom: 18px;
}

.profile-box {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 10px;
  margin-bottom: 18px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: rgba(17,17,22,.7);
}

.avatar {
  width: 42px; height: 42px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  background: linear-gradient(135deg, var(--red), var(--red-2));
  font-weight: 800;
}
.avatar.large { width: 52px; height: 52px; }
.profile-name { font-weight: 700; }
.profile-status { color: var(--muted); font-size: 12px; }

.nav {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 10px;
}
.nav-item {
  background: #111116;
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 12px;
  text-align: left;
  padding: 12px 14px;
  cursor: pointer;
}
.nav-item.active {
  background: linear-gradient(135deg, rgba(239,68,68,.2), rgba(153,27,27,.15));
  border-color: rgba(239,68,68,.45);
}

.sidebar-actions {
  margin-top: 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.primary-btn, .secondary-btn {
  border: none;
  border-radius: 10px;
  padding: 11px 14px;
  font-weight: 700;
  cursor: pointer;
}
.primary-btn {
  background: linear-gradient(135deg, var(--red), var(--red-2));
  color: white;
  box-shadow: 0 10px 25px var(--shadow);
}
.secondary-btn {
  background: #121216;
  color: var(--text);
  border: 1px solid var(--border);
}
.small { padding: 9px 12px; font-size: 13px; }

.main-panel {
  flex: 1;
  border-radius: 18px;
  background: rgba(20,20,26,.88);
  border: 1px solid var(--border);
  overflow: hidden;
}

.topbar {
  display: flex; justify-content: space-between; align-items: center;
  gap: 16px; padding: 22px 28px 18px; border-bottom: 1px solid var(--border);
}
.title-group h1 { margin: 0; font-size: 30px; letter-spacing: .05em; }
.title-group span { display: block; margin-top: 6px; color: var(--muted); font-size: 13px; }
.top-actions {
  display: flex; align-items: center; gap: 8px;
}
.top-actions input {
  width: 260px; background: #111116; border: 1px solid var(--border); border-radius: 10px;
  color: var(--text); padding: 10px 12px;
}

.view { display: none; padding: 20px 28px 28px; }
.view.active { display: block; }

.story-row {
  display: flex; gap: 18px; overflow-x: auto; padding: 16px 0 20px; border-bottom: 1px solid var(--border);
}
.story-item {
  min-width: 110px; text-align: center; color: var(--muted); font-size: 12px;
}
.story-ring {
  width: 66px; height: 66px; border-radius: 50%; margin: 0 auto 8px;
  background: linear-gradient(135deg, #f87171, #7f1d1d); padding: 2px;
}
.story-ring .inner {
  width: 100%; height: 100%; border-radius: 50%; background: #1f1f2c;
  display: flex; align-items: center; justify-content: center; font-weight: 800;
}

.composer {
  margin-top: 18px; padding: 18px; border: 1px solid var(--border); border-radius: 16px; background: rgba(17,17,22,.8);
}
.composer textarea, .composer input {
  width: 100%; background: #111116; border: 1px solid var(--border); color: var(--text); border-radius: 12px; padding: 12px 14px; margin-bottom: 10px;
}
.actions { display: flex; gap: 10px; }

.content-grid {
  margin-top: 22px; display: grid; grid-template-columns: 1.5fr 1fr; gap: 18px;
}
.card {
  background: rgba(17,17,22,.9); border: 1px solid var(--border); border-radius: 16px; padding: 18px;
}
.card h3, .panel-header h3 { margin: 0 0 12px; }
.list, .notif-list, .group-grid, .voice-grid, .reel-grid {
  display: flex; flex-direction: column; gap: 12px;
}
.entry {
  background: rgba(29,29,35,.9); border: 1px solid var(--border); border-radius: 12px; padding: 12px 14px;
}
.entry-header { display: flex; justify-content: space-between; gap: 10px; margin-bottom: 8px; }
.entry-title { font-weight: 700; }
.entry-meta { color: var(--muted); font-size: 12px; }
.entry p { margin: 6px 0 0; color: #d7dee8; font-size: 14px; }
.call-tag {
  display: inline-block; margin-top: 10px; font-size: 11px; padding: 4px 8px; border-radius: 999px;
  background: rgba(34,197,94,.12); border: 1px solid rgba(34,197,94,.5); color: #8ef0a7;
}

.dm-shell {
  display: grid; grid-template-columns: 280px 1fr; min-height: 560px; gap: 16px;
}
.dm-list-panel, .dm-chat-panel {
  background: rgba(17,17,22,.9); border: 1px solid var(--border); border-radius: 16px; padding: 14px;
}
.dm-item {
  width: 100%; background: rgba(29,29,35,.9); border: 1px solid var(--border); padding: 10px 12px; border-radius: 10px; color: var(--text); cursor: pointer; text-align: left; margin-bottom: 8px;
}
.dm-header {
  border-bottom: 1px solid var(--border); padding-bottom: 12px; margin-bottom: 12px; font-weight: 700; font-size: 18px;
}
.dm-messages {
  min-height: 380px; display: flex; flex-direction: column; gap: 10px; overflow: auto; padding-right: 6px;
}
.message {
  max-width: 70%; padding: 10px 12px; border-radius: 12px; background: rgba(29,29,35,.9); border: 1px solid var(--border);
}
.message.mine { margin-left: auto; background: rgba(239,68,68,.12); border-color: rgba(239,68,68,.35); }
.dm-composer { display: flex; gap: 10px; margin-top: 14px; }
.dm-composer input { flex: 1; background: #111116; border: 1px solid var(--border); color: var(--text); padding: 10px 12px; border-radius: 10px; }

.panel-header {
  display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-bottom: 16px;
}
.group-grid, .voice-grid, .reel-grid {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px;
}
.group-card, .voice-card, .reel-card, .notif-item {
  background: rgba(17,17,22,.9); border: 1px solid var(--border); border-radius: 14px; padding: 16px;
}
.badge {
  display: inline-block; padding: 5px 8px; border-radius: 999px; background: rgba(239,68,68,.12); border: 1px solid rgba(239,68,68,.4); color: #f4a5a5; font-size: 11px; margin-bottom: 8px;
}
.voice-card .status-good { color: #8ef0a7; }
.reel-card video, .reel-card .video-placeholder {
  width: 100%; height: 180px; border-radius: 10px; background: linear-gradient(135deg, #2a0d0d, #17171d); display: flex; align-items: center; justify-content: center; color: #f7d3d3; font-weight: 700; border: 1px solid var(--border);
}
.notif-item { display: flex; justify-content: space-between; gap: 12px; }

@media (max-width: 1000px) {
  .app-shell { flex-direction: column; }
  .sidebar { width: 100%; }
  .content-grid, .dm-shell { grid-template-columns: 1fr; }
  .topbar { flex-direction: column; align-items: flex-start; }
  .top-actions { width: 100%; }
  .top-actions input { width: 100%; }
}
