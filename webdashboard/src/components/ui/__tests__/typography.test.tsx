import { render, screen } from '@testing-library/react';
import React from 'react';

import { describe, it, expect } from 'vitest';

import { Typography, GradientText } from '../typography';

describe('Typography Component', () => {
  it('renders default paragraph element', () => {
    render(<Typography>Standard text</Typography>);

    const el = screen.getByText('Standard text');
    expect(el.tagName).toBe('P');
  });

  it('renders hero variant as h1 with strong typography classes', () => {
    render(<Typography variant="hero">Hero Heading</Typography>);

    const el = screen.getByText('Hero Heading');
    expect(el.tagName).toBe('H1');
    expect(el.className).toContain('font-black');
  });

  it('renders tagline variant as p with muted style', () => {
    render(<Typography variant="tagline">Tagline subtitle</Typography>);

    const el = screen.getByText('Tagline subtitle');
    expect(el.tagName).toBe('P');
    expect(el.className).toContain('font-medium');
  });

  it('supports gradient color variant', () => {
    render(<Typography color="gradient">Gradient Title</Typography>);

    const el = screen.getByText('Gradient Title');
    expect(el.className).toContain('bg-clip-text');
    expect(el.className).toContain('text-transparent');
  });
});

describe('GradientText Component', () => {
  it('renders span with gradient text clipping', () => {
    render(<GradientText>Nginx Proxy Manager</GradientText>);

    const el = screen.getByText('Nginx Proxy Manager');
    expect(el.tagName).toBe('SPAN');
    expect(el.className).toContain('bg-clip-text');
    expect(el.className).toContain('text-transparent');
    expect(el.className).toContain('font-extrabold');
  });

  it('allows custom gradient from/to stops', () => {
    render(
      <GradientText from="from-rose-500" to="to-amber-500">
        Custom Colors
      </GradientText>
    );

    const el = screen.getByText('Custom Colors');
    expect(el.className).toContain('from-rose-500');
    expect(el.className).toContain('to-amber-500');
  });
});
