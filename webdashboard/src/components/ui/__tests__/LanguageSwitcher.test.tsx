import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

import { LanguageSwitcher } from '../LanguageSwitcher';

// Mock useLanguage hook
const mockChangeLanguage = vi.fn().mockResolvedValue(undefined);

vi.mock('@/hooks/use-language', () => ({
  useLanguage: () => ({
    currentLanguage: 'vi',
    currentOption: { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳' },
    languages: [
      { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳' },
      { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸' },
      { code: 'zh', name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
    ],
    changeLanguage: mockChangeLanguage,
    userPreference: 'vi',
  }),
}));

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

const renderWithProviders = (ui: React.ReactElement) => {
  const queryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      {ui}
    </QueryClientProvider>
  );
};

describe('LanguageSwitcher', () => {
  beforeEach(() => {
    mockChangeLanguage.mockClear();
  });

  it('renders trigger button with current language', () => {
    renderWithProviders(<LanguageSwitcher />);

    const trigger = screen.getByTestId('language-switcher-trigger');
    expect(trigger).toBeInTheDocument();
    expect(trigger).toHaveTextContent('Tiếng Việt');
  });

  it('opens dropdown on click', async () => {
    renderWithProviders(<LanguageSwitcher />);

    const trigger = screen.getByTestId('language-switcher-trigger');
    fireEvent.click(trigger);

    await waitFor(() => {
      expect(screen.getByTestId('language-option-en')).toBeInTheDocument();
      expect(screen.getByTestId('language-option-vi')).toBeInTheDocument();
      expect(screen.getByTestId('language-option-zh')).toBeInTheDocument();
    });
  });

  it('calls changeLanguage when option is clicked', async () => {
    renderWithProviders(<LanguageSwitcher />);

    // Open dropdown
    fireEvent.click(screen.getByTestId('language-switcher-trigger'));

    await waitFor(() => {
      expect(screen.getByTestId('language-option-en')).toBeInTheDocument();
    });

    // Click English option
    fireEvent.click(screen.getByTestId('language-option-en'));

    await waitFor(() => {
      expect(mockChangeLanguage).toHaveBeenCalledWith('en');
    });
  });

  it('renders in compact mode with globe icon', () => {
    renderWithProviders(<LanguageSwitcher compact />);

    const trigger = screen.getByTestId('language-switcher-trigger');
    expect(trigger).toBeInTheDocument();
    // In compact mode, should not show language name
    expect(trigger).not.toHaveTextContent('Tiếng Việt');
  });

  it('highlights current language in dropdown', async () => {
    renderWithProviders(<LanguageSwitcher />);

    fireEvent.click(screen.getByTestId('language-switcher-trigger'));

    await waitFor(() => {
      const viOption = screen.getByTestId('language-option-vi');
      expect(viOption).toHaveClass('bg-primary-50');
    });
  });
});
