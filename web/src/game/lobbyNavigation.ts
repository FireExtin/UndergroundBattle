import { useEffect, useState } from 'react';

export type LobbyPage = 'home' | 'play' | 'collection';

function currentPage(): LobbyPage {
  const hash = location.hash;
  if (hash === '#collection') return 'collection';
  if (hash === '#play') return 'play';
  if (hash === '#home') return 'home';
  return new URL(location.href).searchParams.has('invite') ? 'play' : 'home';
}

export function navigateLobby(page: LobbyPage) {
  const url = new URL(location.href);
  url.hash = page;
  if (url.href !== location.href) history.pushState({}, '', url);
  window.dispatchEvent(new Event('popstate'));
}

export function useLobbyPage() {
  const [page, setPage] = useState(currentPage);
  useEffect(() => {
    const update = () => setPage(currentPage());
    window.addEventListener('hashchange', update);
    window.addEventListener('popstate', update);
    return () => {
      window.removeEventListener('hashchange', update);
      window.removeEventListener('popstate', update);
    };
  }, []);
  return page;
}
