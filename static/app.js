const state = {
  stories: [],
  searchHistory: [],
  calls: []
};

function formatDurationMs(ms) {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) return `${hours} saat ${minutes} dk ${seconds} sn`;
  if (minutes > 0) return `${minutes} dk ${seconds} sn`;
  return `${seconds} sn`;
}

function formatDateTime(value) {
  const date = new Date(value);
  return `${date.toLocaleDateString('tr-TR')} ${date.toLocaleTimeString('tr-TR', { hour: '2-digit', minute: '2-digit', second: '2-digit' })}`;
}

async function fetchState() {
  const response = await fetch('/api/state');
  const data = await response.json();
  state.stories = data.stories || [];
  state.searchHistory = data.search_history || [];
  state.calls = data.call_sessions || [];
  renderStories();
  renderSearchHistory();
  renderCalls();
}

function renderStories() {
  const row = document.getElementById('storyRow');
  row.innerHTML = '';

  state.stories.forEach((story) => {
    const item = document.createElement('div');
    item.className = 'story-item';

    const ring = document.createElement('div');
    ring.className = 'story-ring';

    const inner = document.createElement('div');
    inner.className = 'inner';
    inner.textContent = story.author.slice(0, 1).toUpperCase();

    ring.appendChild(inner);

    const label = document.createElement('div');
    label.textContent = story.author;

    item.appendChild(ring);
    item.appendChild(label);
    row.appendChild(item);
  });
}

function renderSearchHistory() {
  const list = document.getElementById('searchHistoryList');
  list.innerHTML = '';

  state.searchHistory.forEach((entry) => {
    const item = document.createElement('div');
    item.className = 'entry';
    item.innerHTML = `
      <div class="entry-header">
        <div class="entry-title">${entry.entity_label} · ${entry.scope.toUpperCase()}</div>
        <div class="entry-meta">${formatDurationMs(entry.duration_ms)}</div>
      </div>
      <p><strong>Sorgu:</strong> ${entry.query}</p>
      <p><strong>Başlangıç:</strong> ${formatDateTime(entry.started_at)}</p>
      <p><strong>Bitiş:</strong> ${formatDateTime(entry.ended_at)}</p>
      <p><strong>Sonuç:</strong> ${entry.result_count}</p>
    `;
    list.appendChild(item);
  });
}

function renderCalls() {
  const list = document.getElementById('callHistoryList');
  list.innerHTML = '';

  state.calls.forEach((call) => {
    const item = document.createElement('div');
    item.className = 'entry';
    item.innerHTML = `
      <div class="entry-header">
        <div class="entry-title">${call.call_type.toUpperCase()} · ${call.room_id}</div>
        <div class="entry-meta">${formatDurationMs(call.duration_ms)}</div>
      </div>
      <p><strong>Başlangıç:</strong> ${formatDateTime(call.started_at)}</p>
      <p><strong>Bitiş:</strong> ${call.ended_at ? formatDateTime(call.ended_at) : 'Devam ediyor'}</p>
      <p><strong>Katılımcılar:</strong> ${call.participants.join(', ')}</p>
      <span class="call-tag">${call.status}</span>
    `;
    list.appendChild(item);
  });
}

async function publishStory() {
  const caption = document.getElementById('storyText').value.trim();
  const mediaUrl = document.getElementById('storyImage').value.trim();
  const payload = {
    caption,
    media_url: mediaUrl || null,
    media_type: mediaUrl ? 'image' : 'text'
  };

  const response = await fetch('/api/stories', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });

  if (response.ok) {
    document.getElementById('storyText').value = '';
    document.getElementById('storyImage').value = '';
    await fetchState();
  }
}

async function login() {
  const username = prompt('Kullanıcı adını gir:');
  const email = prompt('E-posta adresini gir:');
  if (!username || !email) return;

  await fetch('/api/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, email })
  });

  await fetchState();
}

document.getElementById('publishStoryBtn').addEventListener('click', publishStory);
document.getElementById('loginBtn').addEventListener('click', login);

fetchState();
