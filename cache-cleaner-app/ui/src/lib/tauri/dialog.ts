import { ask, message } from '@tauri-apps/plugin-dialog'

/**
 * Show a confirmation dialog
 * @param title - Dialog title
 * @param message - Dialog message
 * @returns true if user confirmed, false otherwise
 */
export async function confirm(title: string, msg: string): Promise<boolean> {
  return ask(msg, { title, kind: 'warning' })
}

/**
 * Show an info message dialog
 */
export async function showInfo(title: string, msg: string): Promise<void> {
  await message(msg, { title, kind: 'info' })
}

/**
 * Show an error message dialog
 */
export async function showError(title: string, msg: string): Promise<void> {
  await message(msg, { title, kind: 'error' })
}
