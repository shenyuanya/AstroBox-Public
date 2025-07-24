import { useNavDirection } from '@/animation/NavDirectionContext';
import { NAV_EVENT } from '@/animation/navigationEvent';
import { useRouter } from 'next/router';

export function useAnimatedRouter() {
  const router = useRouter();
  const { setDirection } = useNavDirection();

  const startNav = (cb: () => Promise<any> | void) => {
    let callResolve: () => void;
    let callReject: (e: unknown) => void;
    const promise = new Promise<void>((resolve, reject) => {
      callResolve = resolve;
      callReject = reject;
    });

    const call = () => {
      const finish = () => {
        router.events.off('routeChangeComplete', finish);
        callResolve();
      };
      router.events.on('routeChangeComplete', finish);
      try {
        const r = cb();
        if (r && typeof (r as any).catch === 'function') {
          (r as Promise<any>).catch(err => {
            router.events.off('routeChangeComplete', finish);
            callReject(err);
          });
        }
      } catch (e) {
        router.events.off('routeChangeComplete', finish);
        callReject(e);
      }
      return promise;
    };
    window.dispatchEvent(new CustomEvent(NAV_EVENT, { detail: call }));
    return promise;
  };

  return {
    ...router,
    push: (...props: Parameters<typeof router.push>) => {
      setDirection('forward');
      return startNav(() => router.push(...props));
    },
    pushWithoutAnimation: (...props: Parameters<typeof router.push>) => {
      setDirection('none');
      return router.push(...props);
    },
    back: () => {
      setDirection('back');
      return startNav(() => Promise.resolve(router.back()));
    },
    backWithoutAnimation: () => {
      setDirection('none');
      return router.back();
    },
    replace: (...props: Parameters<typeof router.replace>) => {
      setDirection('forward');
      return startNav(() => router.replace(...props));
    },
  };
}
