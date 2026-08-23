export const useTranslation = () => ({
  t: (key: string, options?: any) => {
    // Support both i18next signatures:
    // - t(key, { defaultValue })
    // - t(key, defaultValueString)
    if (typeof options === 'string') return options;

    // Check if we're running in zh-CN locale (set by vitest.setup.ts)
    const lng = globalThis.localStorage?.getItem('i18nextLng');

    // Chinese translation map for commonly-tested keys
    if (lng === 'zh-CN') {
      const zhMap: Record<string, string> = {
        'settings:thinking.high': '推理: 高',
        'settings:thinking.xhigh': '推理: 超高',
        'settings:thinking.unsupported': '推理: 不支持',
      };
      if (zhMap[key]) return zhMap[key];
    }

    return options?.defaultValue ?? key;
  },
  i18n: {
    changeLanguage: () => Promise.resolve(),
    language: 'en-US',
  },
});

export const initReactI18next = {
  type: '3rdParty',
  init: () => undefined,
};

export default {
  useTranslation,
  initReactI18next,
};







