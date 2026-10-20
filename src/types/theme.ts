export const themes = [
{ id: 'neon-pink', name: 'Neon Pink', accent: '#fe007f', description: 'Твоя ночная частота' },
{ id: 'arctic-blue', name: 'Arctic Blue', accent: '#55d5fc', description: 'Чистое звучание севера' },
{ id: 'forest-gold', name: 'Forest Gold', accent: '#f1ca7c', description: 'Тёплый ритм природы' },
{ id: 'crimson-night', name: 'Crimson Night', accent: '#e7272d', description: 'Энергия после полуночи' },
] as const
export type ThemeId = typeof themes[number]['id']
export function isTheme(value: unknown): value is ThemeId { return themes.some(t => t.id === value) }
