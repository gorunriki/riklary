<script>
  import { onMount } from 'svelte';

  export let onSelectBook; // Callback untuk mengabari App.svelte buku apa yang dipilih

  let books = [];
  let loading = true;
  let error = null;
  let searchQuery = '';
  let selectedSubject = 'Semua';

  onMount(async () => {
    try {
      const res = await fetch('/api/books');
      if (!res.ok) throw new Error('Gagal mengambil data buku');
      books = await res.json();
    } catch (err) {
      error = err.message;
      console.error(err);
    } finally {
      loading = false;
    }
  });

  // Mengumpulkan kategori/tag subjek unik untuk tab filter
  $: subjects = ['Semua', ...new Set(books.flatMap(b => b.subjects || []))];

  // Memfilter buku berdasarkan pencarian dan kategori terpilih
  $: filteredBooks = books.filter(book => {
    const matchesSearch = book.title.toLowerCase().includes(searchQuery.toLowerCase()) || 
                          book.author.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesSubject = selectedSubject === 'Semua' || 
                           (book.subjects && book.subjects.includes(selectedSubject));

    return matchesSearch && matchesSubject;
  });

  // Mengecek apakah buku memiliki riwayat progres membaca
  function hasProgress(fileName) {
    return localStorage.getItem(`progress-${fileName}`) !== null;
  }
</script>

<div class="library-container">
  <header class="library-header">
    <h1>Koleksi Buku</h1>
    <p class="subtitle">{books.length} e-book tersedia</p>
  </header>

  <!-- Kolom Pencarian -->
  <div class="search-box">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="search-icon">
      <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
    </svg>
    <input 
      type="text" 
      placeholder="Cari judul atau penulis..." 
      bind:value={searchQuery}
    />
  </div>

  <!-- Scroll Kategori Horizontal -->
  {#if !loading && subjects.length > 1}
    <div class="categories-scroll">
      {#each subjects as subject}
        <button 
          class="category-pill" 
          class:active={selectedSubject === subject}
          on:click={() => selectedSubject = subject}
        >
          {subject}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Main Grid Buku -->
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Memuat rak buku Anda...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <p>⚠️ {error}</p>
      <button on:click={() => window.location.reload()}>Coba Lagi</button>
    </div>
  {:else if filteredBooks.length === 0}
    <div class="empty-state">
      <p>Tidak ada buku yang cocok di rak.</p>
    </div>
  {:else}
    <div class="books-grid">
      {#each filteredBooks as book}
        <div class="book-card" on:click={() => onSelectBook(book)}>
          <div class="book-info">
            <div class="tags-container">
              {#if hasProgress(book.file_name)}
                <span class="tag progress-tag">📖 Dibaca</span>
              {/if}
              {#each book.subjects.slice(0, 2) as subject}
                <span class="tag">{subject}</span>
              {/each}
            </div>
            <h3 class="book-title">{book.title}</h3>
            <p class="book-author">✍️ {book.author}</p>
          </div>
          <div class="card-footer">
            <span class="read-btn">
              {hasProgress(book.file_name) ? 'Lanjutkan' : 'Baca Sekarang'}
            </span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .library-container {
    width: 100%;
  }

  .library-header {
    margin-bottom: 20px;
  }

  .library-header h1 {
    font-size: 26px;
    font-weight: 800;
    color: hsl(var(--text-primary));
  }

  .subtitle {
    font-size: 13px;
    color: hsl(var(--text-secondary));
  }

  /* PENCARIAN */
  .search-box {
    position: relative;
    width: 100%;
    margin-bottom: 16px;
  }

  .search-icon {
    position: absolute;
    left: 14px;
    top: 50%;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    color: hsl(var(--text-secondary));
  }

  input {
    width: 100%;
    padding: 12px 16px 12px 42px;
    background-color: hsl(var(--bg-secondary));
    border: 1px solid hsl(var(--border));
    border-radius: 12px;
    color: hsl(var(--text-primary));
    font-size: 14px;
    outline: none;
    transition: all 0.2s ease;
  }

  input:focus {
    border-color: hsl(var(--accent));
    box-shadow: 0 0 0 3px hsl(var(--accent-light));
  }

  /* KATEGORI SCROLL */
  .categories-scroll {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 8px;
    margin-bottom: 20px;
    scrollbar-width: none;
  }

  .categories-scroll::-webkit-scrollbar {
    display: none;
  }

  .category-pill {
    white-space: nowrap;
    background-color: hsl(var(--bg-secondary));
    border: 1px solid hsl(var(--border));
    color: hsl(var(--text-secondary));
    padding: 8px 14px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .category-pill.active {
    background-color: hsl(var(--accent));
    border-color: hsl(var(--accent));
    color: #ffffff;
  }

  /* KARTU BUKU GRID */
  .books-grid {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .book-card {
    background-color: hsl(var(--bg-secondary));
    border: 1px solid hsl(var(--border));
    border-radius: 16px;
    padding: 18px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    transition: all 0.2s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);
  }

  .book-card:active {
    transform: scale(0.97);
    background-color: hsl(var(--bg-surface));
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 8px;
  }

  .tag {
    font-size: 10px;
    font-weight: 600;
    background-color: hsl(var(--bg-surface));
    color: hsl(var(--text-secondary));
    padding: 3px 8px;
    border-radius: 6px;
  }

  .progress-tag {
    background-color: hsl(var(--accent-light));
    color: hsl(var(--accent));
  }

  .book-title {
    font-size: 16px;
    font-weight: 700;
    line-height: 1.35;
    color: hsl(var(--text-primary));
    margin-bottom: 4px;
  }

  .book-author {
    font-size: 13px;
    color: hsl(var(--text-secondary));
  }

  .card-footer {
    margin-top: 14px;
    display: flex;
    justify-content: flex-end;
  }

  .read-btn {
    font-size: 12px;
    font-weight: 600;
    color: hsl(var(--accent));
  }

  /* LOADING & ERROR */
  .loading-state, .empty-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    color: hsl(var(--text-secondary));
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid hsl(var(--border));
    border-top-color: hsl(var(--accent));
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 12px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-state button {
    margin-top: 12px;
    background-color: hsl(var(--accent));
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 8px;
    cursor: pointer;
  }
</style>
