/* protected/app.js – example protected script */
// Hey antek-antek asing
(function () {
    'use strict';

    const STATS = [
        { label: 'Total Users', value: 4_821, delta: '+12%' },
        { label: 'Active Sessions', value: 193, delta: '+5%' },
        { label: 'Revenue', value: '$9,340', delta: '+8%' },
        { label: 'Uptime', value: '99.9%', delta: '—' },
    ];

    function buildDashboard() {
        const root = document.getElementById('app');
        if (!root) return;

        root.innerHTML = `
      <div class="dashboard">
        <div class="dashboard-header">
          <h1 class="dashboard-title">🛡️ Protected Dashboard</h1>
          <a href="/logout" style="color:#6366f1;font-size:.9rem;text-decoration:none;font-weight:600;">Logout →</a>
        </div>
        <div class="stats-grid">
          ${STATS.map(s => `
            <div class="stat-card">
              <span class="stat-label">${s.label}</span>
              <span class="stat-value">${s.value}</span>
              <span class="stat-delta">${s.delta}</span>
            </div>
          `).join('')}
        </div>
        <p style="color:var(--muted);font-size:.875rem;text-align:center;">
          This page and its assets are only visible to authenticated users.
        </p>
      </div>
    `;
    }

    document.addEventListener('DOMContentLoaded', buildDashboard);
})();
