import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useFontManager } from './useFontManager'
import { useSettings } from './useSettings'

/**
 * 应用初始化组合式函数
 */
export function useAppInitialization(mcpHandler: ReturnType<typeof import('./useMcpHandler').useMcpHandler>) {
  const isInitializing = ref(true)
  const { loadFontConfig } = useFontManager()
  const settings = useSettings()
  const { getCliArgs, checkMcpModeWithArgs, setupMcpEventListener, restorePendingMcpRequest } = mcpHandler

  function runDeferredStartupTasks() {
    window.setTimeout(async () => {
      try {
        await settings.loadWindowConfig()
        await settings.setupWindowFocusListener()
      }
      catch (error) {
        console.warn('延后加载窗口配置失败:', error)
      }
    }, 0)
  }

  /**
   * 检查是否为首次启动
   */
  function checkFirstRun(): boolean {
    // 检查localStorage是否有初始化标记
    const hasInitialized = localStorage.getItem('app-initialized')
    return !hasInitialized
  }

  /**
   * 标记应用已初始化
   */
  function markAsInitialized() {
    localStorage.setItem('app-initialized', 'true')
  }

  /**
   * 初始化应用
   */
  async function initializeApp() {
    try {
      // 检查是否为首次启动
      const isFirstRun = checkFirstRun()
      const cliArgs = await getCliArgs()

      await setupMcpEventListener()
      await restorePendingMcpRequest()

      const { isMcp, mcpContent } = await checkMcpModeWithArgs(cliArgs)
      const isPopupSession = isMcp || mcpHandler.isDaemonMode.value

      // 主题已在useTheme初始化时加载，这里不需要重复加载

      // 并行加载首屏关键配置，避免弹窗被非关键任务阻塞
      await Promise.all([
        loadFontConfig(),
        settings.loadWindowSettings(),
      ])

      // 在MCP模式下，确保前端状态与后端窗口状态同步
      if (isPopupSession) {
        console.log('MCP模式检测到，同步窗口状态...')
        try {
          await settings.syncWindowStateFromBackend()
        }
        catch (error) {
          console.warn('MCP模式状态同步失败，继续初始化:', error)
        }
      }

      if (!isPopupSession) {
        runDeferredStartupTasks()
      }

      // 如果是首次启动，标记已初始化（主题已在上面加载过）
      if (isFirstRun) {
        console.log('检测到首次启动，标记应用已初始化')
        markAsInitialized()
      }

      // 结束初始化状态
      isInitializing.value = false

      // 非守护进程模式下显示窗口；守护进程模式窗口由后端 IPC 控制显示
      if (!mcpHandler.isDaemonMode.value) {
        try {
          await getCurrentWindow().show()
        }
        catch (e) {
          console.warn('显示窗口失败:', e)
        }
      }

      return { isMcp, mcpContent }
    }
    catch (error) {
      console.error('应用初始化失败:', error)
      isInitializing.value = false
      throw error
    }
  }

  return {
    isInitializing,
    initializeApp,
  }
}
