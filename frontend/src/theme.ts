// Runtime light/dark theme management.
// Default follows the OS (prefers-color-scheme); a user override is stored in localStorage
// and applied via the data-theme attribute on <html>, which main.css keys off of.

export type Theme = 'light' | 'dark';

const STORAGE_KEY = 'theme';

export function get_effective_theme(): Theme
{
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === 'light' || saved === 'dark')
    {
        return saved;
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

// Apply the saved override (if any) to <html>. When no override is stored the attribute is
// removed so the OS preference takes over. Call this on page load.
export function apply_theme(): void
{
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === 'light' || saved === 'dark')
    {
        document.documentElement.setAttribute('data-theme', saved);
    }
    else
    {
        document.documentElement.removeAttribute('data-theme');
    }
}

// Flip the effective theme, persist the choice, re-apply, and return the new value.
export function toggle_theme(): Theme
{
    const next: Theme = get_effective_theme() === 'dark' ? 'light' : 'dark';
    localStorage.setItem(STORAGE_KEY, next);
    apply_theme();
    return next;
}
