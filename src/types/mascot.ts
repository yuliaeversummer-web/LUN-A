export const mascotStates = ['welcome', 'listening', 'sleeping', 'curious', 'mute', 'next', 'error', 'idle'] as const
export type MascotState = typeof mascotStates[number]
export type MascotSize = 'compact' | 'medium' | 'large'

export const mascotPoses: Record<MascotState, { src: string; alt: string; caption: string }> = {
  welcome: { src: '/mascot/poses/welcome.png', alt: 'Лунэя приветливо машет рукой', caption: 'Привет, я Лунэя.' },
  listening: { src: '/mascot/poses/listening.png', alt: 'Лунэя в наушниках мечтательно улыбается, прикрыв глаза', caption: 'На твоей частоте.' },
  sleeping: { src: '/mascot/poses/sleeping.png', alt: 'Лунэя спит, положив голову на руки', caption: 'Побудем в тишине.' },
  curious: { src: '/mascot/poses/curious.png', alt: 'Лунэя задумчиво касается подбородка', caption: 'Какое у тебя настроение?' },
  mute: { src: '/mascot/poses/mute.png', alt: 'Лунэя подносит палец к губам, предлагая тишину', caption: 'Микрофон на паузе.' },
  next: { src: '/mascot/poses/next.png', alt: 'Лунэя показывает маленькое неоновое сердечко пальцами', caption: 'Ещё одна любимая волна.' },
  error: { src: '/mascot/poses/error.png', alt: 'Лунэя смущённо улыбается, положив руку за голову', caption: 'Кажется, что-то пошло не так…' },
  idle: { src: '/mascot/poses/idle.png', alt: 'Лунэя обнимает мягкую подушку с лунным котом', caption: 'Я рядом. Не спеши.' },
}
