# 🎨 UI/UX Design Rules

## 1. Design System
- **Components:** Use predefined components in `webdashboard/src/components/ui`. Don't reinvent the wheel.
- **Premium Patterns:** Use premium design patterns (e.g., Combobox/Radix instead of native selects).

## 2. Accessibility & Standards
- **Compliance:** Minimum WCAG 2.1 AA compliance.
- **Interactions:** Mandatory `aria-labels` and explicit keyboard focus states.

## 3. Responsive & Motion
- **Layout:** Mobile-first approach.
- **Animations:** Use subtle transitions (`duration-200`); avoid jarring or over-the-top animations. Implement smooth micro-interactions.

## 4. Edge Cases
- Always implement Skeleton loaders while fetching data.
- Always implement clear, beautiful Empty States when there is no data.
