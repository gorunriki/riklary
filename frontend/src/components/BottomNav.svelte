<script>
  export let activeTab = 'library'; // 'library', 'reader', 'settings'
  export let hasActiveBook = false; // Deteksi jika ada buku yang sedang aktif dibaca
</script>

<nav class="bottom-nav">
  <!-- Tombol Tab Rak Buku -->
  <button 
    class="nav-item" 
    class:active={activeTab === 'library'} 
    on:click={() => activeTab = 'library'}
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="nav-icon">
      <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/>
      <path d="M6 6h10M6 10h10"/>
    </svg>
    <span class="nav-label">Rak</span>
  </button>

  <!-- Tombol Tab Ruang Baca (Hanya bisa diklik jika ada buku aktif) -->
  <button 
    class="nav-item" 
    class:active={activeTab === 'reader'} 
    class:disabled={!hasActiveBook}
    disabled={!hasActiveBook}
    on:click={() => activeTab = 'reader'}
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="nav-icon">
      <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
      <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
    </svg>
    <span class="nav-label">Baca</span>
  </button>

  <!-- Tombol Tab Pengaturan -->
  <button 
    class="nav-item" 
    class:active={activeTab === 'settings'} 
    on:click={() => activeTab = 'settings'}
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="nav-icon">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
    </svg>
    <span class="nav-label">Setelan</span>
  </button>
</nav>

<style>
  .bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    width: 100%;
    height: calc(60px + var(--safe-bottom));
    background-color: hsl(var(--bg-secondary));
    border-top: 1px solid hsl(var(--border));
    display: flex;
    justify-content: space-around;
    align-items: center;
    padding-bottom: var(--safe-bottom);
    z-index: 90;
    transition: background-color 0.3s ease, border-color 0.3s ease;
  }

  .nav-item {
    background: none;
    border: none;
    outline: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: hsl(var(--text-secondary));
    width: 33.33%;
    height: 100%;
    cursor: pointer;
    transition: color 0.2s ease, transform 0.1s ease;
  }

  .nav-item:active:not(.disabled) {
    transform: scale(0.95);
  }

  .nav-icon {
    width: 22px;
    height: 22px;
    margin-bottom: 4px;
    transition: transform 0.2s ease;
  }

  .nav-label {
    font-size: 11px;
    font-weight: 500;
  }

  /* Aktif & Fokus State */
  .nav-item.active {
    color: hsl(var(--accent));
  }

  .nav-item.active .nav-icon {
    transform: translateY(-2px);
    stroke: hsl(var(--accent));
  }

  /* Disabled State (Misal belum membuka buku) */
  .nav-item.disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
