<script>
  import { onMount } from 'svelte';
  import BottomNav from './components/BottomNav.svelte';
  import Library from './components/Library.svelte';
  import Reader from './components/Reader.svelte';

  let activeTab = 'library';
  let currentBook = null;
  let hasActiveBook = false;

  // Pengaturan global membaca
  let globalTheme = 'light';
  let globalFontSize = '100%';
  let globalLineHeight = '1.6';
  let globalFontFamily = 'serif';

  // Penanda apakah setelan dari localStorage sudah selesai dimuat
  let isInitialized = false;

  // Reactively sinkronkan tema data-attribute ke tag <html>
  $: if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', globalTheme);
  }

  onMount(() => {
    // 1. Muat preferensi setelan yang tersimpan
    globalTheme = localStorage.getItem('riklary-theme') || 'light';
    globalFontSize = localStorage.getItem('riklary-font-size') || '100%';
    globalLineHeight = localStorage.getItem('riklary-line-height') || '1.6';
    globalFontFamily = localStorage.getItem('riklary-font-family') || 'serif';
    
    // 2. Muat buku terakhir yang dibuka jika ada
    const savedBook = localStorage.getItem('riklary-active-book');
    if (savedBook) {
      currentBook = JSON.parse(savedBook);
      hasActiveBook = true;
    }

    // 3. Setel flag ke true agar penyimpanan otomatis aktif setelah ini
    isInitialized = true;
  });

  // Menyimpan otomatis setelan ke localStorage HANYA setelah inisialisasi selesai
  $: if (isInitialized && typeof localStorage !== 'undefined') {
    localStorage.setItem('riklary-theme', globalTheme);
    localStorage.setItem('riklary-font-size', globalFontSize);
    localStorage.setItem('riklary-line-height', globalLineHeight);
    localStorage.setItem('riklary-font-family', globalFontFamily);
  }

  function handleBookSelect(book) {
    currentBook = book;
    hasActiveBook = true;
    localStorage.setItem('riklary-active-book', JSON.stringify(book));
    activeTab = 'reader'; // Otomatis berpindah ke tab pembaca
  }
</script>


<main class="app-container">
  {#if activeTab === 'library'}
    <div class="page-content">
      <Library onSelectBook={handleBookSelect} />
    </div>
  {:else if activeTab === 'reader'}
    <div class="page-content reader-page">
<Reader 
        book={currentBook} 
        bind:theme={globalTheme}
        bind:fontSize={globalFontSize}
        bind:lineHeight={globalLineHeight}
        bind:fontFamily={globalFontFamily}
        on:backToLibrary={() => activeTab = 'library'}
      />
    </div>
  {:else if activeTab === 'settings'}
    <div class="page-content settings-page">
      <header class="settings-header">
        <h2>Setelan Aplikasi</h2>
      </header>
      <div class="settings-list">
        <div class="settings-section">
          <h3>Tampilan & Tema</h3>
          <div class="setting-item">
            <span>Tema Warna</span>
            <div class="theme-picker">
              <button class="theme-btn light" class:selected={globalTheme === 'light'} on:click={() => globalTheme = 'light'}>Terang</button>
              <button class="theme-btn sepia" class:selected={globalTheme === 'sepia'} on:click={() => globalTheme = 'sepia'}>Sepia</button>
              <button class="theme-btn dark" class:selected={globalTheme === 'dark'} on:click={() => globalTheme = 'dark'}>Gelap</button>
            </div>
          </div>

          <div class="setting-item">
            <span>Ukuran Font</span>
            <select bind:value={globalFontSize}>
              <option value="80%">Kecil</option>
              <option value="100%">Normal</option>
              <option value="120%">Besar</option>
              <option value="140%">Ekstra Besar</option>
            </select>
          </div>

          <div class="setting-item">
            <span>Jarak Baris</span>
            <select bind:value={globalLineHeight}>
              <option value="1.4">Rapat</option>
              <option value="1.6">Normal</option>
              <option value="1.8">Longgar</option>
              <option value="2.0">Lebar</option>
            </select>
          </div>

          <div class="setting-item">
            <span>Jenis Font</span>
            <select bind:value={globalFontFamily}>
              <option value="serif">Merriweather (Serif)</option>
              <option value="sans-serif">Inter (Sans-Serif)</option>
            </select>
          </div>
        </div>

        <div class="settings-section info-section">
          <h3>Tentang Riklary</h3>
          <p>Riklary adalah pembaca e-book (EPUB) mobile-first minimalis berbasis Rust + Svelte.</p>
          <div class="version">Versi 1.0.0-beta</div>
        </div>
      </div>
    </div>
  {/if}

  <BottomNav bind:activeTab {hasActiveBook} />
</main>

<style>
  .app-container {
    width: 100%;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    padding-bottom: calc(60px + var(--safe-bottom)); /* Jarak cadangan untuk BottomNav */
  }

  .page-content {
    flex: 1;
    width: 100%;
    max-width: 600px; /* Lebar maksimal layar HP modern */
    margin: 0 auto;
    padding: 20px;
    animation: fadeIn 0.2s ease-out;
  }

  .reader-page {
    padding: 0;
    max-width: none; /* Halaman pembaca memakai lebar penuh layar HP */
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* SETTINGS PANEL */
  .settings-header {
    margin-bottom: 24px;
  }
  
  .settings-header h2 {
    font-size: 24px;
    font-weight: 700;
  }

  .settings-section {
    background-color: hsl(var(--bg-secondary));
    border: 1px solid hsl(var(--border));
    border-radius: 16px;
    padding: 20px;
    margin-bottom: 20px;
  }

  .settings-section h3 {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 16px;
    color: hsl(var(--text-primary));
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 0;
    border-bottom: 1px solid hsl(var(--border));
  }

  .setting-item:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .setting-item span {
    font-size: 14px;
    font-weight: 500;
  }

  select {
    background-color: hsl(var(--bg-surface));
    color: hsl(var(--text-primary));
    border: 1px solid hsl(var(--border));
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 13px;
    outline: none;
  }

  .theme-picker {
    display: flex;
    gap: 6px;
  }

  .theme-btn {
    border: 1px solid hsl(var(--border));
    background: hsl(var(--bg-surface));
    color: hsl(var(--text-primary));
    padding: 6px 10px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .theme-btn.selected {
    border-color: hsl(var(--accent));
    background-color: hsl(var(--accent-light));
    color: hsl(var(--accent));
  }

  .info-section p {
    font-size: 13px;
    color: hsl(var(--text-secondary));
    margin-bottom: 12px;
    line-height: 1.6;
  }

  .version {
    font-size: 11px;
    color: hsl(var(--text-secondary));
    opacity: 0.7;
  }
</style>
