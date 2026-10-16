import { useEffect, useRef, useState } from 'react';

type Registration = {
  observer: IntersectionObserver;
  listeners: Map<Element, (near: boolean) => void>;
};
const registrations = new WeakMap<Document, Registration>();

/** Gate image sources, not card rows, so offscreen cards remain keyboard reachable. */
export function useCardVisibility(eager: boolean) {
  const ref = useRef<HTMLSpanElement>(null);
  const [near, setNear] = useState(eager);
  useEffect(() => {
    const element = ref.current;
    if (eager || !element) return;
    // Older browsers retain their native lazy-image behavior.
    if (typeof IntersectionObserver === 'undefined') { setNear(true); return; }
    const document = element.ownerDocument;
    let registration = registrations.get(document);
    if (!registration) {
      const listeners: Registration['listeners'] = new Map();
      const observer = new IntersectionObserver(entries => {
        for (const entry of entries) listeners.get(entry.target)?.(entry.isIntersecting);
      }, { root: null, rootMargin: '180px 0px', threshold: 0 });
      registration = { observer, listeners };
      registrations.set(document, registration);
    }
    registration.listeners.set(element, setNear);
    registration.observer.observe(element);
    return () => {
      registration.listeners.delete(element);
      registration.observer.unobserve(element);
      if (!registration.listeners.size) {
        registration.observer.disconnect();
        registrations.delete(document);
      }
    };
  }, [eager]);
  return { ref, near: eager || near };
}
