<script>
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import JSZip from 'jszip';
  import ePub from 'epubjs';

  // Menyematkan JSZip ke window secara global karena dibutuhkan oleh epub.js
  if (typeof window !== 'undefined') {
    window.JSZip = JSZip;
  }

  const dispatch = createEventDispatcher();

  export let book; // Objek info buku yang dipilih
  export let theme = 'light';
  export let fontSize = '100%';
  export let lineHeight = '1.6';
  export let fontFamily = 'serif';

  let bookDoc = null;
  let rendition = null;
  let viewerElement;

  let showControls = true;
  let showTocSheet = false;
  let showSettingsSheet = false;
  
  let toc = [];
  let currentChapterName = 'Memuat bab...';
  let loadingBook = true;

  // Nilai koordinat untuk gestur geser (swipe)
  let touchStartX = 0;
  let touchEndX = 0;

  // Konfigurasi style tema khusus isi buku ePub
  const THEME_STYLES = {
    light: { body: { background: '#fbfaf7', color: '#2d3135' } },
    dark: { body: { background: '#121826', color: '#cbd5e1' } },
    sepia: { body: { background: '#f4ecd8', color: '#433422' } }
  };

  // Reaktif mengupdate setelan ke epub.js setiap ada perubahan dari dropdown/pilihan
  // Reaktif mengupdate setelan ke epub.js setiap ada perubahan dari dropdown/pilihan
  $: if (rendition) {
    rendition.themes.select(theme);
    rendition.themes.fontSize(fontSize);
    
    const fontVal = fontFamily === 'serif' ? "'Merriweather', serif" : "'Inter', sans-serif";
    
    // Tentukan warna HEX dinamis berdasarkan tema aktif saat ini
    const activeBg = theme === 'dark' ? '#121826' : (theme === 'sepia' ? '#f4ecd8' : '#fbfaf7');
    const activeFg = theme === 'dark' ? '#cbd5e1' : (theme === 'sepia' ? '#433422' : '#2d3135');
    
    // Suntikkan style secara agresif untuk menimpa CSS bawaan e-book yang buruk
    rendition.themes.default({
      body: {
        'padding': '60px 24px 60px 24px !important',
        'background-color': `${activeBg} !important`,
        'color': `${activeFg} !important`
      },
      'p, body, div, span, li': {
        'font-family': `${fontVal} !important`,
        'line-height': `${lineHeight} !important`,
        'background-color': 'transparent !important', /* Paksa kontainer dalam menjadi transparan */
        'color': `${activeFg} !important` /* Paksa warna teks agar terbaca */
      }
    });
  }



  onMount(async () => {
    if (!book) return;

    const bookUrl = `/static/${book.extracted_path}`;
    
    try {
      bookDoc = ePub(bookUrl);
      
      // Render e-book ke wadah dengan opsi spread: 'none' agar pas satu halaman di mobile
      rendition = bookDoc.renderTo(viewerElement, {
        width: '100%',
        height: '100%',
        spread: 'none'
      });

      // Registrasi semua tema warna
      Object.keys(THEME_STYLES).forEach(name => {
        rendition.themes.register(name, THEME_STYLES[name]);
      });

      // Muat progress membaca terakhir jika ada
      const savedProgress = localStorage.getItem(`progress-${book.file_name}`);
      if (savedProgress) {
        await rendition.display(savedProgress);
      } else {
        await rendition.display();
      }

      // Ambil navigasi / Daftar Isi
      bookDoc.loaded.navigation.then(nav => {
        toc = nav.toc;

                if (rendition && rendition.location) {
          const currentHref = rendition.location.start.href;
          const matchedChapter = findChapterByHref(currentHref, toc);
          if (matchedChapter) {
            currentChapterName = matchedChapter.label.trim();
          }
        }
      });

      

      // Update judul bab saat ini ketika halaman berpindah
     rendition.on('relocated', (location) => {
        const cfi = location.start.cfi;
        const currentHref = location.start.href;
        localStorage.setItem(`progress-${book.file_name}`, cfi);
        // Cari judul bab dari file aktif
        if (toc && toc.length > 0) {
          const matchedChapter = findChapterByHref(currentHref, toc);
          if (matchedChapter) {
            currentChapterName = matchedChapter.label.trim();
          } else {
            currentChapterName = book.title;
          }
        } else {
          currentChapterName = book.title;
        }
        loadingBook = false;
      });

    } catch (err) {
      console.error('Gagal memuat e-book:', err);
      loadingBook = false;
    }
  });

  onDestroy(() => {
    if (bookDoc) {
      try {
        bookDoc.destroy();
      } catch (e) {
        console.error(e);
      }
    }
  });

  function prevPage() {
    if (rendition) rendition.prev();
  }

  function nextPage() {
    if (rendition) rendition.next();
  }

  function toggleControls() {
    showControls = !showControls;
  }

  function goToChapter(href) {
    if (rendition) {
      rendition.display(href);
      showTocSheet = false;
      showControls = false;
    }
  }

  // Gestur sentuh (Swipe)
  function handleTouchStart(e) {
    touchStartX = e.changedTouches[0].screenX;
  }

  function handleTouchEnd(e) {
    touchEndX = e.changedTouches[0].screenX;
    handleSwipe();
  }

  function handleSwipe() {
    const swipeDistance = touchEndX - touchStartX;
    if (swipeDistance > 55) {
      prevPage(); // Geser ke kanan -> halaman sebelumnya
    } else if (swipeDistance < -55) {
      nextPage(); // Geser ke kiri -> halaman berikutnya
    }
  }

    // Menyimpan status buka-tutup per volume (key: chapter.href, value: boolean)
  let expandedChapters = {};

  function toggleChapter(chapter) {
    if (chapter.subitems && chapter.subitems.length > 0) {
      // Toggle status dan re-assign objek agar memicu reaktivitas Svelte
      expandedChapters[chapter.href] = !expandedChapters[chapter.href];
      expandedChapters = { ...expandedChapters };
    } else {
      // Jika tidak ada sub-bab, langsung lompat ke halaman tersebut
      goToChapter(chapter.href);
    }
  }

    // Fungsi rekursif untuk mencari bab yang cocok dengan file aktif saat ini
  function findChapterByHref(href, items) {
    if (!items || !href) return null;
    
    // Hilangkan jangkar/anchor (#...) agar perbandingan nama file bersih
    const cleanHref = href.split('#')[0];
    
    for (const item of items) {
      const cleanItemHref = item.href.split('#')[0];
      
      // Jika nama file-nya cocok
      if (cleanHref === cleanItemHref || href.includes(item.href) || item.href.includes(href)) {
        return item;
      }
      
      // Cari ke dalam sub-bab jika ada (rekursif)
      if (item.subitems && item.subitems.length > 0) {
        const found = findChapterByHref(href, item.subitems);
        if (found) return found;
      }
    }
    return null;
  }


</script>

<div class="reader-container" class:immersive={!showControls}>
  
  <!-- Bar Atas (Header Controls) -->
  <header class="reader-header" class:hide={!showControls}>
    <button class="icon-btn" on:click={() => dispatch('backToLibrary')}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="icon">
        <path d="m15 18-6-6 6-6"/>
      </svg>
    </button>
    <div class="header-titles">
      <!-- Tampilkan judul buku di atas jika bab yang aktif berbeda dari judul buku -->
      {#if currentChapterName !== book.title}
        <span class="book-title-tiny">{book.title}</span>
      {/if}
      <span class="chapter-title-bold truncate">{currentChapterName}</span>
    </div>

    <button class="icon-btn" on:click={() => { showTocSheet = true; }}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="icon">
        <line x1="3" x2="21" y1="6" y2="6"/><line x1="3" x2="21" y1="12" y2="12"/><line x1="3" x2="21" y1="18" y2="18"/>
      </svg>
    </button>
  </header>

  <!-- Area Viewer Buku -->
  <div class="viewer-wrapper">
    {#if loadingBook}
      <div class="loading-epub">
        <div class="spinner"></div>
        <p>Menyiapkan lembaran buku...</p>
      </div>
    {/if}
    <div bind:this={viewerElement} class="epub-viewer"></div>
    
    <!-- Transparent Gesture Overlay di atas Iframe -->
    <div 
      class="gesture-overlay"
      on:touchstart={handleTouchStart}
      on:touchend={handleTouchEnd}
    >
      <!-- Ketuk 25% Kiri -> Prev -->
      <div class="tap-zone prev" on:click|stopPropagation={prevPage}></div>
      <!-- Ketuk 50% Tengah -> Menu -->
      <div class="tap-zone center" on:click|stopPropagation={toggleControls}></div>
      <!-- Ketuk 25% Kanan -> Next -->
      <div class="tap-zone next" on:click|stopPropagation={nextPage}></div>
    </div>
  </div>

  <!-- Bar Bawah (Quick Settings Control) -->
  <footer class="reader-footer" class:hide={!showControls}>
    <button class="footer-btn" on:click={() => { showSettingsSheet = true; }}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="icon">
        <path d="M4 12h16M4 6h16M4 18h16"/>
      </svg>
      Tampilan & Font
    </button>
  </footer>

  <!-- LACI DAFTAR ISI (TOC SHEET) -->
  {#if showTocSheet}
    <div class="sheet-overlay" on:click={() => showTocSheet = false}></div>
    <div class="bottom-sheet">
      <div class="sheet-handle"></div>
      <div class="sheet-header">
        <h3>Daftar Bab</h3>
        <button class="close-sheet-btn" on:click={() => showTocSheet = false}>Tutup</button>
      </div>
      <ul class="toc-list">
        {#each toc as chapter}
          <!-- Header Volume/Bab Utama -->
          <li 
            class="toc-item volume-header" 
            on:click={() => toggleChapter(chapter)}
          >
            <!-- Chevron Indikator Buka/Tutup -->
            <span class="chevron">
              {#if chapter.subitems && chapter.subitems.length > 0}
                {expandedChapters[chapter.href] ? '▼' : '▶'}
              {/if}
            </span>
            <span class="volume-title">{chapter.label.trim()}</span>
          </li>
          
          <!-- Daftar Sub-bab (Hanya muncul jika volume di-expand/dibuka) -->
          {#if chapter.subitems && chapter.subitems.length > 0 && expandedChapters[chapter.href]}
            {#each chapter.subitems as sub}
              <li class="toc-item sub-item" on:click={() => goToChapter(sub.href)}>
                <span class="sub-prefix">↳</span> {sub.label.trim()}
              </li>
            {/each}
          {/if}
        {/each}
      </ul>



    </div>
  {/if}

  <!-- LACI PENGATURAN CEPAT (DISPLAY SETTINGS SHEET) -->
  {#if showSettingsSheet}
    <div class="sheet-overlay" on:click={() => showSettingsSheet = false}></div>
    <div class="bottom-sheet">
      <div class="sheet-handle"></div>
      <div class="sheet-header">
        <h3>Pengaturan Tampilan</h3>
        <button class="close-sheet-btn" on:click={() => showSettingsSheet = false}>Tutup</button>
      </div>
      
      <div class="quick-settings">
        <!-- Tema -->
        <div class="setting-row">
          <span class="label">Tema Warna</span>
          <div class="theme-options">
            <button class="theme-pill bg-light" class:selected={theme === 'light'} on:click={() => theme = 'light'}>Terang</button>
            <button class="theme-pill bg-sepia" class:selected={theme === 'sepia'} on:click={() => theme = 'sepia'}>Sepia</button>
            <button class="theme-pill bg-dark" class:selected={theme === 'dark'} on:click={() => theme = 'dark'}>Gelap</button>
          </div>
        </div>

        <!-- Ukuran Font -->
        <div class="setting-row">
          <span class="label">Ukuran Huruf</span>
          <div class="font-controls">
            <button class="adjust-btn" on:click={() => { if(fontSize !== '80%') fontSize = (parseInt(fontSize)-20) + '%' }}>A-</button>
            <span class="font-val">{fontSize}</span>
            <button class="adjust-btn" on:click={() => { if(fontSize !== '140%') fontSize = (parseInt(fontSize)+20) + '%' }}>A+</button>
          </div>
        </div>

        <!-- Jarak Baris -->
        <div class="setting-row">
          <span class="label">Jarak Baris</span>
          <select bind:value={lineHeight}>
            <option value="1.4">Rapat (1.4)</option>
            <option value="1.6">Normal (1.6)</option>
            <option value="1.8">Longgar (1.8)</option>
            <option value="2.0">Lebar (2.0)</option>
          </select>
        </div>

        <!-- Jenis Font -->
        <div class="setting-row">
          <span class="label">Gaya Font</span>
          <select bind:value={fontFamily}>
            <option value="serif">Merriweather (Serif)</option>
            <option value="sans-serif">Inter (Sans)</option>
          </select>
        </div>
      </div>
    </div>
  {/if}

</div>

<style>
  .reader-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: hsl(var(--bg-primary));
    display: flex;
    flex-direction: column;
    z-index: 100;
    transition: background-color 0.3s ease;
  }


  /* BAR ATAS (HEADER) */
  .reader-header {
    height: 56px;
    background-color: hsl(var(--bg-secondary));
    border-bottom: 1px solid hsl(var(--border));
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px;
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    z-index: 20;
    transition: transform 0.3s cubic-bezier(0.1, 0.76, 0.55, 0.94);
  }

  .reader-header.hide {
    transform: translateY(-56px);
  }

  /* Judul Ganda di Header */
  .header-titles {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    max-width: 65%;
    text-align: center;
  }

  .book-title-tiny {
    font-size: 10px;
    color: hsl(var(--text-secondary));
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .chapter-title-bold {
    font-size: 13px;
    font-weight: 700;
    color: hsl(var(--text-primary));
    width: 100%;
  }


  .icon-btn {
    background: none;
    border: none;
    outline: none;
    color: hsl(var(--text-primary));
    padding: 8px;
    cursor: pointer;
  }

  .icon {
    width: 22px;
    height: 22px;
  }

  /* WADAH VIEWER & GESTURE */
  .viewer-wrapper {
    flex: 1;
    position: relative;
    width: 100%;
    height: 100%;
  }

  .epub-viewer {
    width: 100%;
    height: 100%;
  }

  .gesture-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    z-index: 15;
  }

  .tap-zone {
    height: 100%;
    cursor: pointer;
  }

  .tap-zone.prev {
    width: 25%;
  }

  .tap-zone.center {
    width: 50%;
  }

  .tap-zone.next {
    width: 25%;
  }

  /* BAR BAWAH (FOOTER) */
  .reader-footer {
    height: 50px;
    background-color: hsl(var(--bg-secondary));
    border-top: 1px solid hsl(var(--border));
    display: flex;
    justify-content: center;
    align-items: center;
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    z-index: 20;
    padding-bottom: var(--safe-bottom);
    transition: transform 0.3s cubic-bezier(0.1, 0.76, 0.55, 0.94);
  }

  .reader-footer.hide {
    transform: translateY(calc(50px + var(--safe-bottom)));
  }

  .footer-btn {
    background: none;
    border: none;
    display: flex;
    align-items: center;
    gap: 8px;
    color: hsl(var(--text-primary));
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  /* LACI (BOTTOM SHEETS) */
  .sheet-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .sheet-header h3 {
    font-size: 16px;
    font-weight: 700;
  }

  .close-sheet-btn {
    background: none;
    border: none;
    color: hsl(var(--accent));
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .toc-list {
    list-style: none;
    padding-bottom: 30px;
  }

  .toc-item {
    padding: 12px 14px;
    font-size: 14px;
    border-bottom: 1px solid hsl(var(--border));
    cursor: pointer;
    border-radius: 8px;
    color: hsl(var(--text-primary));
    transition: background-color 0.2s;
  }

  .toc-item:active {
    background-color: hsl(var(--bg-surface));
  }

    /* Style khusus untuk Volume Header */
  .toc-item.volume-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    background-color: hsl(var(--bg-surface));
    margin-top: 8px;
    border-left: 3px solid hsl(var(--accent));
    border-radius: 4px;
    font-size: 14px;
  }

    .chevron {
    display: inline-block;
    width: 14px;
    font-size: 9px;
    color: hsl(var(--text-secondary));
    text-align: center;
  }

    .volume-title {
    flex: 1;
  }

  /* Style khusus untuk Sub-bab */
  .toc-item.sub-item {
    padding-left: 28px;
    font-size: 13px;
    color: hsl(var(--text-primary));
    opacity: 0.9;
    border-bottom: 1px dashed hsl(var(--border));
  }

  .sub-prefix {
    color: hsl(var(--accent));
    margin-right: 4px;
    font-weight: bold;
  }


  /* QUICK SETTINGS DI LACI */
  .quick-settings {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 12px;
    border-bottom: 1px solid hsl(var(--border));
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
  }

  .theme-options {
    display: flex;
    gap: 6px;
  }

  .theme-pill {
    border: 1px solid hsl(var(--border));
    padding: 6px 12px;
    border-radius: 8px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  .theme-pill.bg-light { background-color: #ffffff; color: #111827; }
  .theme-pill.bg-sepia { background-color: #f4ecd8; color: #5b4636; }
  .theme-pill.bg-dark { background-color: #1f2937; color: #f9fafb; }

  .theme-pill.selected {
    border-color: hsl(var(--accent));
    box-shadow: 0 0 0 2px hsl(var(--accent-light));
  }

  .font-controls {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .adjust-btn {
    border: 1px solid hsl(var(--border));
    background-color: hsl(var(--bg-surface));
    color: hsl(var(--text-primary));
    width: 32px;
    height: 32px;
    border-radius: 50%;
    font-size: 14px;
    font-weight: bold;
    cursor: pointer;
  }

  .font-val {
    font-size: 13px;
    font-weight: 600;
  }

  select {
    background-color: hsl(var(--bg-surface));
    color: hsl(var(--text-primary));
    border: 1px solid hsl(var(--border));
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
    outline: none;
  }

  /* LOADING STATE EPUB */
  .loading-epub {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: hsl(var(--text-secondary));
    z-index: 16;
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid hsl(var(--border));
    border-top-color: hsl(var(--accent));
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
