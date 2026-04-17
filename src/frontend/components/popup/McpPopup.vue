<script setup lang="ts">
import type { McpRequest } from '../../types/popup'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import PopupActions from './PopupActions.vue'
import PopupContent from './PopupContent.vue'
import PopupInput from './PopupInput.vue'

interface AppConfig {
  theme: string
  window: {
    alwaysOnTop: boolean
    width: number
    height: number
    fixed: boolean
  }
  audio: {
    enabled: boolean
    url: string
  }
  reply: {
    enabled: boolean
    prompt: string
  }
}

interface Props {
  request: McpRequest | null
  appConfig: AppConfig
  mockMode?: boolean
  testMode?: boolean
}

interface Emits {
  response: [response: any]
  cancel: []
  themeChange: [theme: string]
  openMainLayout: []
  toggleAlwaysOnTop: []
  toggleAudioNotification: []
  updateAudioUrl: [url: string]
  testAudio: []
  stopAudio: []
  testAudioError: [error: any]
  updateWindowSize: [size: { width: number, height: number, fixed: boolean }]
}

interface PopupInputData {
  userInput: string
  selectedOptions: string[]
  draggedImages: string[]
}

interface PopupInputExpose {
  statusText?: string
  updateData: (data: Partial<PopupInputData>) => void
  handleQuoteMessage: (messageContent: string) => void
  getCurrentData: () => PopupInputData
}

const props = withDefaults(defineProps<Props>(), {
  mockMode: false,
  testMode: false,
})

const emit = defineEmits<Emits>()

// 使用消息提示
const message = useMessage()

// 响应式状态
const loading = ref(false)
const submitting = ref(false)
const selectedOptions = ref<string[]>([])
const userInput = ref('')
const draggedImages = ref<string[]>([])
const inputRef = ref<PopupInputExpose | null>(null)
const countdownRemainingMs = ref(0)
const countdownDeadline = ref<number | null>(null)
const countdownPaused = ref(false)
let countdownTimer: ReturnType<typeof window.setInterval> | null = null
let loadingFrame: number | null = null

// 继续回复配置
const continueReplyEnabled = ref(true)
const continuePrompt = ref('请按照最佳实践继续')
const defaultTimeoutMs = 600000

// 计算属性
const isVisible = computed(() => !!props.request)
const hasOptions = computed(() => (props.request?.predefined_options?.length ?? 0) > 0)
const timeoutMs = computed(() => props.request?.timeout_ms ?? defaultTimeoutMs)
const hasCountdown = computed(() => timeoutMs.value > 0)
const projectName = computed(() => props.request?.project_name || '当前项目')
const countdownText = computed(() => formatCountdown(countdownRemainingMs.value))
const isCountdownWarning = computed(() => countdownRemainingMs.value <= 10000)
const countdownStatusText = computed(() => {
  if (countdownPaused.value) {
    return '计时已暂停'
  }
  return isCountdownWarning.value ? '即将过时，若无响应会自动重新发起' : '当前请求倒计时进行中'
})
const canSubmit = computed(() => {
  if (hasOptions.value) {
    return selectedOptions.value.length > 0 || userInput.value.trim().length > 0 || draggedImages.value.length > 0
  }
  return userInput.value.trim().length > 0 || draggedImages.value.length > 0
})

// 获取输入组件的状态文本
const inputStatusText = computed(() => {
  return inputRef.value?.statusText || '等待输入...'
})

function syncInputStateFromChild() {
  const latest = inputRef.value?.getCurrentData()
  if (!latest) {
    return
  }

  userInput.value = latest.userInput
  selectedOptions.value = latest.selectedOptions
  draggedImages.value = latest.draggedImages
}

// 加载继续回复配置
async function loadReplyConfig() {
  try {
    const config = await invoke('get_reply_config')
    if (config) {
      const replyConfig = config as any
      continueReplyEnabled.value = replyConfig.enable_continue_reply ?? true
      continuePrompt.value = replyConfig.continue_prompt ?? '请按照最佳实践继续'
    }
  }
  catch (error) {
    console.log('加载继续回复配置失败，使用默认值:', error)
  }
}

function formatCountdown(remainingMs: number) {
  const totalSeconds = Math.max(0, Math.ceil(remainingMs / 1000))
  const minutes = String(Math.floor(totalSeconds / 60)).padStart(2, '0')
  const seconds = String(totalSeconds % 60).padStart(2, '0')
  return `${minutes}:${seconds}`
}

function clearCountdownTimer() {
  if (countdownTimer) {
    window.clearInterval(countdownTimer)
    countdownTimer = null
  }
}

async function syncTimerState(paused: boolean, remainingMs: number) {
  if (!props.request || props.mockMode)
    return

  try {
    await invoke('sync_popup_timer_state', {
      request: props.request,
      remainingMs: Math.max(1000, remainingMs),
      paused,
    })
  }
  catch (error) {
    console.error('同步弹窗计时状态失败:', error)
  }
}

function handleToggleCountdown() {
  if (countdownPaused.value) {
    void resumeCountdown()
  }
  else {
    void pauseCountdown()
  }
}

function startCountdown() {
  clearCountdownTimer()

  if (!props.request || !hasCountdown.value) {
    countdownRemainingMs.value = 0
    countdownDeadline.value = null
    countdownPaused.value = false
    return
  }

  countdownPaused.value = false
  countdownDeadline.value = Date.now() + timeoutMs.value
  countdownRemainingMs.value = timeoutMs.value

  countdownTimer = window.setInterval(() => {
    updateCountdown()
  }, 1000)
}

function updateCountdown() {
  if (countdownPaused.value) {
    return
  }

  if (!countdownDeadline.value) {
    countdownRemainingMs.value = 0
    return
  }

  const remaining = countdownDeadline.value - Date.now()
  countdownRemainingMs.value = Math.max(0, remaining)

  if (remaining <= 0) {
    clearCountdownTimer()
    void handleTimeout()
  }
}

async function pauseCountdown() {
  if (!hasCountdown.value || countdownPaused.value)
    return

  updateCountdown()
  clearCountdownTimer()
  countdownPaused.value = true
  countdownDeadline.value = null
  await syncTimerState(true, countdownRemainingMs.value)
}

async function resumeCountdown() {
  if (!hasCountdown.value || !countdownPaused.value)
    return

  countdownPaused.value = false
  countdownDeadline.value = Date.now() + countdownRemainingMs.value
  countdownTimer = window.setInterval(() => {
    updateCountdown()
  }, 1000)
  await syncTimerState(false, countdownRemainingMs.value)
}

async function resetCountdown() {
  if (!hasCountdown.value)
    return

  clearCountdownTimer()
  countdownPaused.value = false
  countdownRemainingMs.value = timeoutMs.value
  countdownDeadline.value = Date.now() + timeoutMs.value
  countdownTimer = window.setInterval(() => {
    updateCountdown()
  }, 1000)
  await syncTimerState(false, timeoutMs.value)
}

// 监听配置变化（当从设置页面切换回来时）
watch(() => props.appConfig.reply, (newReplyConfig) => {
  if (newReplyConfig) {
    continueReplyEnabled.value = newReplyConfig.enabled
    continuePrompt.value = newReplyConfig.prompt
  }
}, { deep: true, immediate: true })

// Telegram事件监听器
let telegramUnlisten: (() => void) | null = null

function schedulePopupReady() {
  if (loadingFrame) {
    cancelAnimationFrame(loadingFrame)
  }

  loadingFrame = window.requestAnimationFrame(() => {
    loading.value = false
    loadingFrame = null
  })
}

// 监听请求变化
watch(() => props.request, (newRequest) => {
  clearCountdownTimer()
  if (loadingFrame) {
    cancelAnimationFrame(loadingFrame)
    loadingFrame = null
  }

  if (newRequest) {
    resetForm()
    loading.value = true
    loadReplyConfig()
    startCountdown()
    schedulePopupReady()
  }
  else {
    countdownRemainingMs.value = 0
    countdownDeadline.value = null
    countdownPaused.value = false
  }
}, { immediate: true })

// 设置Telegram事件监听
async function setupTelegramListener() {
  try {
    telegramUnlisten = await listen('telegram-event', (event) => {
      handleTelegramEvent(event.payload as any)
    })
  }
  catch (error) {
    console.error('设置Telegram事件监听器失败:', error)
  }
}

// 处理Telegram事件
function handleTelegramEvent(event: any) {
  switch (event.type) {
    case 'option_toggled':
      handleOptionToggle(event.option)
      break
    case 'text_updated':
      handleTextUpdate(event.text)
      break
    case 'continue_pressed':
      handleContinue()
      break
    case 'send_pressed':
      handleSubmit()
      break
    default:
      console.warn('未知 Telegram 事件类型:', event.type)
  }
}

// 处理选项切换
function handleOptionToggle(option: string) {
  const index = selectedOptions.value.indexOf(option)
  if (index > -1) {
    // 取消选择
    selectedOptions.value.splice(index, 1)
  }
  else {
    // 添加选择
    selectedOptions.value.push(option)
  }

  // 同步到PopupInput组件
  if (inputRef.value) {
    inputRef.value.updateData({ selectedOptions: selectedOptions.value })
  }
}

// 处理文本更新
function handleTextUpdate(text: string) {
  userInput.value = text

  // 同步到PopupInput组件
  if (inputRef.value) {
    inputRef.value.updateData({ userInput: text })
  }
}

// 组件挂载时设置监听器和加载配置
onMounted(() => {
  loadReplyConfig()
  setupTelegramListener()
})

// 组件卸载时清理监听器
onUnmounted(() => {
  clearCountdownTimer()
  if (loadingFrame) {
    cancelAnimationFrame(loadingFrame)
    loadingFrame = null
  }
  if (telegramUnlisten) {
    telegramUnlisten()
  }
})

// 重置表单
function resetForm() {
  selectedOptions.value = []
  userInput.value = ''
  draggedImages.value = []
  submitting.value = false
  countdownPaused.value = false
}

async function handleTimeout() {
  if (submitting.value || !props.request) {
    return
  }

  submitting.value = true

  try {
    const response = {
      user_input: null,
      selected_options: [],
      images: [],
      metadata: {
        timestamp: new Date().toISOString(),
        request_id: props.request.id || null,
        source: 'popup_timeout',
      },
    }

    if (props.mockMode) {
      await new Promise(resolve => setTimeout(resolve, 500))
      message.warning('倒计时已结束，模拟自动重发')
    }
    else {
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('发送超时响应失败:', error)
    message.error('超时处理失败，请重试')
    submitting.value = false
    startCountdown()
  }
}

// 处理提交
async function handleSubmit() {
  syncInputStateFromChild()

  if (!canSubmit.value || submitting.value)
    return

  submitting.value = true

  try {
    // 使用新的结构化数据格式
    const response = {
      user_input: userInput.value.trim() || null,
      selected_options: selectedOptions.value,
      images: draggedImages.value.map(imageData => ({
        data: imageData.split(',')[1], // 移除 data:image/png;base64, 前缀
        media_type: 'image/png',
        filename: null,
      })),
      metadata: {
        timestamp: new Date().toISOString(),
        request_id: props.request?.id || null,
        source: 'popup',
      },
    }

    // 如果没有任何有效内容，设置默认用户输入
    if (!response.user_input && response.selected_options.length === 0 && response.images.length === 0) {
      response.user_input = '用户确认继续'
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('模拟响应发送成功')
    }
    else {
      // 实际发送响应
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('提交响应失败:', error)
    message.error('提交失败，请重试')
  }
  finally {
    submitting.value = false
  }
}

// 处理输入更新
function handleInputUpdate(data: { userInput: string, selectedOptions: string[], draggedImages: string[] }) {
  userInput.value = data.userInput
  selectedOptions.value = data.selectedOptions
  draggedImages.value = data.draggedImages
}

// 处理图片添加 - 移除重复逻辑，避免双重添加
function handleImageAdd(_image: string) {
  // 这个函数现在只是为了保持接口兼容性，实际添加在PopupInput中完成
}

// 处理图片移除
function handleImageRemove(index: number) {
  draggedImages.value.splice(index, 1)
}

// 处理继续按钮点击
async function handleContinue() {
  if (submitting.value)
    return

  submitting.value = true

  try {
    // 使用新的结构化数据格式
    const response = {
      user_input: continuePrompt.value,
      selected_options: [],
      images: [],
      metadata: {
        timestamp: new Date().toISOString(),
        request_id: props.request?.id || null,
        source: 'popup_continue',
      },
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('继续请求发送成功')
    }
    else {
      // 实际发送继续请求
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('发送继续请求失败:', error)
    message.error('继续请求失败，请重试')
  }
  finally {
    submitting.value = false
  }
}

// 处理引用消息
function handleQuoteMessage(messageContent: string) {
  if (inputRef.value) {
    inputRef.value.handleQuoteMessage(messageContent)
  }
}

// 处理增强按钮点击
async function handleEnhance() {
  if (submitting.value)
    return

  submitting.value = true

  try {
    // 构建增强prompt
    const enhancePrompt = `Use the following prompt to optimize and enhance the context of the content in 《》, and return the enhanced result by calling the tool 'cunzhi' after completion.Here is an instruction that I'd like to give you, but it needs to be improved. Rewrite and enhance this instruction to make it clearer, more specific, less ambiguous, and correct any mistakes. Reply immediately with your answer, even if you're not sure. Consider the context of our conversation history when enhancing the prompt. Reply with the following format:

### BEGIN RESPONSE ###
Here is an enhanced version of the original instruction that is more specific and clear:
<augment-enhanced-prompt>enhanced prompt goes here</augment-enhanced-prompt>

### END RESPONSE ###

Here is my original instruction:

《${userInput.value.trim()}》`

    // 使用新的结构化数据格式
    const response = {
      user_input: enhancePrompt,
      selected_options: [],
      images: [],
      metadata: {
        timestamp: new Date().toISOString(),
        request_id: props.request?.id || null,
        source: 'popup_enhance',
      },
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('增强请求发送成功')
    }
    else {
      // 实际发送增强请求
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('发送增强请求失败:', error)
    message.error('增强请求失败，请重试')
  }
  finally {
    submitting.value = false
  }
}
</script>

<template>
  <div v-if="isVisible" class="flex flex-col flex-1">
    <!-- 内容区域 - 可滚动 -->
    <div class="flex-1 overflow-y-auto scrollbar-thin">
      <!-- 消息内容 - 允许选中 -->
      <div class="mx-2 mt-2 mb-1 px-4 py-3 bg-black-100 rounded-lg select-text" data-guide="popup-content">
        <PopupContent :request="request" :loading="loading" :current-theme="props.appConfig.theme" @quote-message="handleQuoteMessage" />
      </div>
      <!-- 输入和选项 - 允许选中 -->
      <div class="px-4 pb-3 bg-black select-text">
        <PopupInput
          ref="inputRef" :request="request" :loading="loading" :submitting="submitting"
          @update="handleInputUpdate" @image-add="handleImageAdd" @image-remove="handleImageRemove"
        />
      </div>
    </div>

    <!-- 底部操作栏 - 固定在底部 -->
    <div class="flex-shrink-0 bg-black-100 border-t-2 border-black-200" data-guide="popup-actions">
      <PopupActions
        :request="request" :loading="loading" :submitting="submitting" :can-submit="canSubmit"
        :continue-reply-enabled="continueReplyEnabled" :input-status-text="inputStatusText"
        :countdown-text="hasCountdown ? countdownText : ''" :countdown-paused="countdownPaused"
        :has-countdown="hasCountdown"
        @submit="handleSubmit" @continue="handleContinue" @enhance="handleEnhance"
        @toggle-countdown="handleToggleCountdown" @reset-countdown="resetCountdown"
      />
    </div>
  </div>
</template>
