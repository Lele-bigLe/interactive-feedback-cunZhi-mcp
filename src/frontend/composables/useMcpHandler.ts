import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * MCP处理组合式函数
 */
export function useMcpHandler() {
  const mcpRequest = ref(null)
  const showMcpPopup = ref(false)
  const isDaemonMode = ref(false)
  let mcpEventUnlisten: (() => void) | null = null

  function getRequestId(request: any) {
    return typeof request?.id === 'string' ? request.id : null
  }

  async function applyMcpRequest(request: any) {
    const incomingRequestId = getRequestId(request)
    const currentRequestId = getRequestId(mcpRequest.value)

    if (showMcpPopup.value && incomingRequestId && currentRequestId === incomingRequestId) {
      return
    }

    await showMcpDialog(request)
  }

  async function getCliArgs() {
    try {
      return await invoke('get_cli_args') as any
    }
    catch (error) {
      console.error('获取 CLI 参数失败:', error)
      return {}
    }
  }

  /**
   * 统一的MCP响应处理
   */
  async function handleMcpResponse(response: any) {
    try {
      // 通过Tauri命令发送响应
      await invoke('send_mcp_response', { response })

      if (isDaemonMode.value) {
        // 守护进程模式：保存窗口位置后隐藏窗口，不退出
        showMcpPopup.value = false
        mcpRequest.value = null
        await invoke('save_window_position')
        await getCurrentWindow().hide()
      }
      else {
        // 普通模式：退出应用
        await invoke('exit_app')
      }
    }
    catch (error) {
      console.error('MCP响应处理失败:', error)
    }
  }

  /**
   * 统一的MCP取消处理
   */
  async function handleMcpCancel() {
    try {
      // 发送取消信息
      await invoke('send_mcp_response', { response: 'CANCELLED' })

      if (isDaemonMode.value) {
        // 守护进程模式：保存窗口位置后隐藏窗口，不退出
        showMcpPopup.value = false
        mcpRequest.value = null
        await invoke('save_window_position')
        await getCurrentWindow().hide()
      }
      else {
        // 普通模式：退出应用
        await invoke('exit_app')
      }
    }
    catch (error) {
      // 静默处理MCP取消错误
      console.error('MCP取消处理失败:', error)
    }
  }

  /**
   * 显示MCP弹窗
   */
  async function showMcpDialog(request: any) {
    if (request?.force_frontend_popup) {
      mcpRequest.value = request
      showMcpPopup.value = true

      try {
        await invoke('play_notification_sound')
      }
      catch (error) {
        console.error('播放音频通知失败:', error)
      }

      return
    }

    // 获取Telegram配置，检查是否需要隐藏前端弹窗
    let shouldShowFrontendPopup = true
    try {
      const telegramConfig = await invoke('get_telegram_config')
      // 如果Telegram启用且配置了隐藏前端弹窗，则不显示前端弹窗
      if (telegramConfig && (telegramConfig as any).enabled && (telegramConfig as any).hide_frontend_popup) {
        shouldShowFrontendPopup = false
        console.log('🔕 根据Telegram配置，隐藏前端弹窗')
      }
    }
    catch (error) {
      console.error('获取Telegram配置失败:', error)
      // 配置获取失败时，保持默认行为（显示弹窗）
    }

    // 根据配置决定是否显示前端弹窗
    if (shouldShowFrontendPopup) {
      // 设置请求数据和显示状态
      mcpRequest.value = request
      showMcpPopup.value = true
    }
    else {
    }

    // 播放音频通知（无论是否显示弹窗都播放）
    try {
      await invoke('play_notification_sound')
    }
    catch (error) {
      console.error('播放音频通知失败:', error)
    }

    // 启动Telegram同步（无论是否显示弹窗都启动）
    try {
      if (request?.message) {
        await invoke('start_telegram_sync', {
          message: request.message,
          predefinedOptions: request.predefined_options || [],
          isMarkdown: request.is_markdown || false,
        })
      }
    }
    catch (error) {
      console.error('启动Telegram同步失败:', error)
    }
  }

  /**
   * 检查MCP模式
   */
  async function checkMcpMode() {
    return checkMcpModeWithArgs(await getCliArgs())
  }

  async function checkMcpModeWithArgs(args: any) {
    try {
      // 检测守护进程模式
      if (args && (args as any).daemon) {
        isDaemonMode.value = true
        console.log('🔄 守护进程模式已激活')
      }

      if (args && (args as any).mcp_request) {
        // 读取MCP请求文件
        const content = await invoke('read_mcp_request', { filePath: (args as any).mcp_request })

        if (content) {
          await applyMcpRequest(content)
        }
        return { isMcp: true, mcpContent: content }
      }
    }
    catch (error) {
      console.error('检查MCP模式失败:', error)
    }
    return { isMcp: false, mcpContent: null }
  }

  /**
   * 设置MCP事件监听器
   */
  async function setupMcpEventListener() {
    try {
      if (mcpEventUnlisten) {
        return
      }

      mcpEventUnlisten = await listen('mcp-request', (event) => {
        void applyMcpRequest(event.payload)
      })
    }
    catch (error) {
      console.error('设置MCP事件监听器失败:', error)
    }
  }

  async function restorePendingMcpRequest() {
    try {
      const pendingRequest = await invoke('consume_pending_mcp_request') as any | null
      if (pendingRequest) {
        await applyMcpRequest(pendingRequest)
      }
    }
    catch (error) {
      console.error('恢复待处理弹窗请求失败:', error)
    }
  }

  function cleanupMcpEventListener() {
    if (mcpEventUnlisten) {
      mcpEventUnlisten()
      mcpEventUnlisten = null
    }
  }

  return {
    mcpRequest,
    showMcpPopup,
    isDaemonMode,
    handleMcpResponse,
    handleMcpCancel,
    showMcpDialog,
    getCliArgs,
    checkMcpMode,
    checkMcpModeWithArgs,
    setupMcpEventListener,
    restorePendingMcpRequest,
    cleanupMcpEventListener,
  }
}
