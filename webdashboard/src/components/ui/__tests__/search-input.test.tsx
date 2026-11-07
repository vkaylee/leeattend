import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { describe, it, expect, vi } from 'vitest';

import { SearchInput, SearchTriggerButton } from '../search-input';

describe('SearchInput Component', () => {
  it('renders input with placeholder and default search icon', () => {
    render(<SearchInput placeholder="Search anything..." />);

    const input = screen.getByPlaceholderText('Search anything...');
    expect(input).toHaveAttribute('type', 'text');
  });

  it('renders shortcut badge when shortcut prop is provided and no value', () => {
    render(<SearchInput placeholder="Search..." shortcut="⌘K" value="" />);

    expect(screen.getByText('⌘K')).toBeInTheDocument();
  });

  it('hides shortcut and shows clear button when input has value', () => {
    const handleClear = vi.fn();
    render(
      <SearchInput
        onClear={handleClear}
        placeholder="Search..."
        shortcut="⌘K"
        value="query"
      />
    );

    expect(screen.queryByText('⌘K')).not.toBeInTheDocument();
    const clearBtn = screen.getByRole('button', { name: /clear search input/i });
    expect(clearBtn).toBeInTheDocument();

    fireEvent.click(clearBtn);
    expect(handleClear).toHaveBeenCalledTimes(1);
  });

  it('supports size variants', () => {
    const { container } = render(<SearchInput size="lg" />);
    expect(container.firstChild).toHaveClass('h-11');
  });

  it('handles loading state with spinner', () => {
    const { container } = render(<SearchInput loading />);
    expect(container.querySelector('.animate-spin')).toBeInTheDocument();
  });
});

describe('SearchTriggerButton Component', () => {
  it('renders trigger button with placeholder and shortcut badge', () => {
    const handleClick = vi.fn();
    render(
      <SearchTriggerButton
        onClick={handleClick}
        placeholder="Quick search..."
        shortcut="⌘K"
      />
    );

    const button = screen.getByRole('button');
    expect(button).toBeInTheDocument();
    expect(screen.getByText('Quick search...')).toBeInTheDocument();
    expect(screen.getByText('⌘K')).toBeInTheDocument();

    fireEvent.click(button);
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
