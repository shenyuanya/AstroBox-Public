"use client";
import { createContext, ReactNode, useContext, useState } from 'react';
import en_US from './en.json';
import zh_CN from './zh.json';
import zh_HK from './zh_HK.json';
import zh_HX from './zh_HX.json';

const resources = { zh_CN, en_US, zh_HK, zh_HX } as const;
type Lang = keyof typeof resources;

const STORAGE_KEY = "language";

interface I18nContextProps {
  lang: Lang;
  langs: Lang[];
  setLang: (lang: Lang) => void;
  t: (key: string | undefined | null) => string;
}

const I18nContext = createContext<I18nContextProps>({
  lang: 'en_US',
  langs: Object.keys(resources) as Lang[],
  setLang: () => {},
  t: (key) => (key ?? ''),
});

function getDefaultLang() {
  try {
    if (typeof localStorage !== 'undefined') {
      const lang = localStorage.getItem(STORAGE_KEY);
      if (lang && lang in resources) {
        return lang as Lang;
      }
    }
  } catch (error) {
    console.error('[i18n] getDefaultLang() failed:', error);
  }
  const browserLang = typeof navigator !== 'undefined' ? navigator.language.replaceAll("-", "_") : '';
  if (browserLang in resources) {
    return browserLang as Lang;
  }
  return 'en_US';
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, _setLang] = useState<Lang>(getDefaultLang());

  const t = (key: string | undefined | null): string => {
    if (!key || typeof key !== 'string') {
      if (process.env.NODE_ENV !== 'production') {
        console.warn('[i18n] t() called with invalid key:', key);
      }
      return key ?? '';
    }
    const obj = resources[lang] as Record<string, any>;
    const result = key.split('.').reduce((acc, cur) => acc?.[cur], obj);
    return typeof result === 'string' ? result : key;
  };

  const setLang = (lang: Lang) => {
    localStorage.setItem(STORAGE_KEY, lang);
    _setLang(lang);
  }

  return (
    <I18nContext.Provider value={{ lang, setLang, t, langs: Object.keys(resources) as Lang[] }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  return useContext(I18nContext);
}