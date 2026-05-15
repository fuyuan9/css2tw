/**
 * Generic TS Dynamic Patterns
 */

// String interpolation
const getBaseClasses = (variant: string) => `btn btn-${variant} shadow-sm`;

// Runtime class builders
function composeClass(...args: any[]) {
    return args.filter(Boolean).join(' ');
}

const isActive = true;
const size = 'lg';

// Usage examples that should be detected as dynamic if used in UI contexts
const classes1 = getBaseClasses('primary');
const classes2 = composeClass('container', isActive && 'active', `size-${size}`);

// Helper utilities
const theme = {
    primary: 'text-blue-500',
    secondary: 'text-gray-500'
};

const getThemeClass = (key: keyof typeof theme) => theme[key];

const activeTheme = getThemeClass('primary');

console.log(classes1, classes2, activeTheme);

// Custom wrapper patterns
const myStyles = {
    wrapper: 'p-4 m-2',
    inner: 'flex items-center'
};

export const styles = myStyles;
