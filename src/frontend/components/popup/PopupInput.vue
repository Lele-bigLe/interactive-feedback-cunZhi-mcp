<script setup lang="ts">
import type { CustomPrompt, McpRequest } from '../../types/popup'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import type { InputInst } from 'naive-ui'
import { useMessage } from 'naive-ui'
import Sortable from 'sortablejs'
import { computed, nextTick, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useKeyboard } from '../../composables/useKeyboard'

interface Props {
  request: McpRequest | null
  loading?: boolean
  submitting?: boolean
}

interface Emits {
  update: [data: {
    userInput: string
    selectedOptions: string[]
    draggedImages: string[]
  }]
  imageAdd: [image: string]
  imageRemove: [index: number]
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  submitting: false,
})

const emit = defineEmits<Emits>()

// 响应式数据
const userInput = ref('')
const selectedOptions = ref<string[]>([])
const uploadedImages = ref<string[]>([])
const textareaRef = ref<InputInst | null>(null)

// 自定义prompt相关状态
const customPrompts = ref<CustomPrompt[]>([])
const customPromptEnabled = ref(true)

// 移除条件性prompt状态管理，直接使用prompt的current_state

// 分离普通prompt和条件性prompt
const normalPrompts = computed(() =>
  customPrompts.value.filter(prompt => prompt.type === 'normal' || !prompt.type),
)

const conditionalPrompts = computed(() =>
  customPrompts.value.filter(prompt => prompt.type === 'conditional'),
)

// 拖拽排序相关状态
const promptContainer = ref<HTMLElement | null>(null)
const sortablePrompts = shallowRef<CustomPrompt[]>([])
let dragSortFrame: number | null = null
let sortableInstance: Sortable | null = null
let emitUpdateTimer: number | null = null
let imeRepairTimer: number | null = null
let focusInputFrame: number | null = null

function destroySortable() {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
}

function handlePromptReorder(oldIndex?: number, newIndex?: number) {
  if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) {
    return
  }

  const newList = [...sortablePrompts.value]
  const [movedItem] = newList.splice(oldIndex, 1)
  newList.splice(newIndex, 0, movedItem)

  sortablePrompts.value = newList

  const conditionalPromptsList = customPrompts.value.filter(prompt => prompt.type === 'conditional')
  customPrompts.value = [...sortablePrompts.value, ...conditionalPromptsList]
  savePromptOrder()
}

// 使用键盘快捷键 composable
const { pasteShortcut } = useKeyboard()

const message = useMessage()

// 计算属性
const hasOptions = computed(() => (props.request?.predefined_options?.length ?? 0) > 0)
const requestOptions = computed(() => props.request?.predefined_options ?? [])
const hasQuickTemplates = computed(() => customPromptEnabled.value && sortablePrompts.value.length > 0)
const hasContextAppend = computed(() => customPromptEnabled.value && conditionalPrompts.value.length > 0)
const canSubmit = computed(() => {
  const hasOptionsSelected = selectedOptions.value.length > 0
  const hasInputText = userInput.value.trim().length > 0
  const hasImages = uploadedImages.value.length > 0

  if (hasOptions.value) {
    return hasOptionsSelected || hasInputText || hasImages
  }
  return hasInputText || hasImages
})

// 工具栏状态文本
const statusText = computed(() => {
  // 检查是否有任何输入内容
  const hasInput = selectedOptions.value.length > 0
    || uploadedImages.value.length > 0
    || userInput.value.trim().length > 0

  // 如果有任何输入内容，返回空字符串让 PopupActions 显示快捷键
  if (hasInput) {
    return ''
  }

  return '等待输入...'
})

// 发送更新事件
function emitUpdate() {
  emit('update', {
    userInput: userInput.value + generateConditionalContent(),
    selectedOptions: [...selectedOptions.value],
    draggedImages: [...uploadedImages.value],
  })
}

function emitUpdateImmediately() {
  if (emitUpdateTimer) {
    clearTimeout(emitUpdateTimer)
    emitUpdateTimer = null
  }

  emitUpdate()
}

function queueEmitUpdate(delay = 120) {
  if (emitUpdateTimer) {
    clearTimeout(emitUpdateTimer)
  }

  emitUpdateTimer = window.setTimeout(() => {
    emitUpdateTimer = null
    emitUpdate()
  }, delay)
}

function handleUserInputChange(value: string) {
  userInput.value = value
  queueEmitUpdate()
}

// 处理选项变化
function handleOptionChange(option: string, checked: boolean) {
  if (checked) {
    selectedOptions.value.push(option)
  }
  else {
    const idx = selectedOptions.value.indexOf(option)
    if (idx > -1)
      selectedOptions.value.splice(idx, 1)
  }
  emitUpdateImmediately()
}

// 处理选项切换（整行点击）
function handleOptionToggle(option: string) {
  const idx = selectedOptions.value.indexOf(option)
  if (idx > -1) {
    selectedOptions.value.splice(idx, 1)
  }
  else {
    selectedOptions.value.push(option)
  }
  emitUpdateImmediately()
}

// 移除了所有拖拽和上传组件相关的代码

function handleImagePaste(event: ClipboardEvent) {
  const items = event.clipboardData?.items
  let hasImage = false

  if (items) {
    for (const item of Array.from(items)) {
      if (item.type.includes('image')) {
        hasImage = true
        const file = item.getAsFile()
        if (file) {
          handleImageFiles([file])
        }
      }
    }
  }

  if (hasImage) {
    event.preventDefault()
  }
}

async function handleImageFiles(files: FileList | File[]): Promise<void> {
  for (const file of Array.from(files)) {
    if (file.type.startsWith('image/')) {
      try {
        const base64 = await fileToBase64(file)

        if (!uploadedImages.value.includes(base64)) {
          uploadedImages.value.push(base64)
          message.success(`图片 ${file.name} 已添加`)
          emitUpdateImmediately()
        }
        else {
          message.warning(`图片 ${file.name} 已存在`)
        }
      }
      catch (error) {
        console.error('图片处理失败:', error)
        message.error(`图片 ${file.name} 处理失败`)
        throw error
      }
    }
  }
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

function removeImage(index: number) {
  uploadedImages.value.splice(index, 1)
  emit('imageRemove', index)
  emitUpdateImmediately()
}

// 移除自定义图片预览功能，改用 Naive UI 的内置预览

// 加载自定义prompt配置
async function loadCustomPrompts() {
  try {
    const config = await invoke('get_custom_prompt_config')
    if (config) {
      const promptConfig = config as any

      customPrompts.value = (promptConfig.prompts || []).sort((a: CustomPrompt, b: CustomPrompt) => a.sort_order - b.sort_order)
      customPromptEnabled.value = promptConfig.enabled ?? true

      sortablePrompts.value = [...normalPrompts.value]

      if (sortablePrompts.value.length > 1) {
        initializeDragSort()
      }
      else {
        destroySortable()
      }
    }
  }
  catch (error) {
    console.error('PopupInput: 加载自定义prompt失败:', error)
  }
}

// 处理自定义prompt点击
function handlePromptClick(prompt: CustomPrompt) {
  // 如果prompt内容为空或只有空格，直接清空输入框
  if (!prompt.content || prompt.content.trim() === '') {
    userInput.value = ''
    emitUpdateImmediately()
    return
  }

  const insertMode = userInput.value.trim() ? 'append' : 'replace'
  insertPromptContent(prompt.content, insertMode)
}

// 处理引用消息内容
function handleQuoteMessage(messageContent: string) {
  const insertMode = userInput.value.trim() ? 'append' : 'replace'
  insertPromptContent(messageContent, insertMode)
}

function focusInputToEnd() {
  if (focusInputFrame) {
    cancelAnimationFrame(focusInputFrame)
  }

  nextTick(() => {
    const inputElement = getInputElement()
    if (!inputElement) {
      return
    }

    inputElement.focus()
    focusInputFrame = window.requestAnimationFrame(() => {
      focusInputFrame = null
      try {
        if (typeof inputElement.setSelectionRange === 'function') {
          inputElement.setSelectionRange(inputElement.value.length, inputElement.value.length)
        }
      }
      catch (error) {
        console.warn('设置光标位置失败:', error)
      }
    })
  })
}

// 插入prompt内容
function insertPromptContent(content: string, mode: 'replace' | 'append' = 'replace') {
  if (mode === 'replace') {
    userInput.value = content
  }
  else {
    userInput.value = userInput.value.trim() + (userInput.value.trim() ? '\n\n' : '') + content
  }

  focusInputToEnd()
  queueEmitUpdate(24)
}

// 处理条件性prompt开关变化
async function handleConditionalToggle(promptId: string, value: boolean) {
  // 先更新本地状态
  const prompt = customPrompts.value.find(p => p.id === promptId)
  if (prompt) {
    prompt.current_state = value
  }

  // 保存到后端
  try {
    await invoke('update_conditional_prompt_state', {
      promptId,
      newState: value,
    })
    emitUpdateImmediately()
  }
  catch (error) {
    console.error('保存条件性prompt状态失败:', error)
    message.error(`保存设置失败: ${(error as any)?.message}` || error)

    // 回滚本地状态
    if (prompt) {
      prompt.current_state = !value
    }
    emitUpdateImmediately()
  }
}

// 生成条件性prompt的追加内容
function generateConditionalContent(): string {
  const conditionalTexts: string[] = []

  conditionalPrompts.value.forEach((prompt) => {
    const isEnabled = prompt.current_state ?? false
    const template = isEnabled ? prompt.template_true : prompt.template_false

    if (template && template.trim()) {
      conditionalTexts.push(template.trim())
    }
  })

  return conditionalTexts.length > 0 ? `\n\n${conditionalTexts.join('\n')}` : ''
}

// 获取条件性prompt的自适应描述
function getConditionalDescription(prompt: CustomPrompt): string {
  const isEnabled = prompt.current_state ?? false
  const template = isEnabled ? prompt.template_true : prompt.template_false

  // 如果有对应状态的模板，显示模板内容，否则显示原始描述
  if (template && template.trim()) {
    return template.trim()
  }

  return prompt.description || ''
}

async function initializeDragSort() {
  await nextTick()
  if (dragSortFrame) {
    cancelAnimationFrame(dragSortFrame)
  }

  if (!promptContainer.value || sortablePrompts.value.length < 2) {
    destroySortable()
    return
  }

  dragSortFrame = window.requestAnimationFrame(() => {
    if (!promptContainer.value || sortablePrompts.value.length < 2) {
      destroySortable()
      dragSortFrame = null
      return
    }

    destroySortable()
    sortableInstance = Sortable.create(promptContainer.value, {
      animation: 200,
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      handle: '.drag-handle',
      forceFallback: true,
      fallbackTolerance: 3,
      onEnd: evt => handlePromptReorder(evt.oldIndex, evt.newIndex),
    })
    dragSortFrame = null
  })
}

// 保存prompt排序
async function savePromptOrder() {
  try {
    const promptIds = sortablePrompts.value.map(p => p.id)
    await invoke('update_custom_prompt_order', { promptIds })
    message.success('排序已保存')
  }
  catch (error) {
    console.error('保存排序失败:', error)
    message.error('保存排序失败')
    // 重新加载以恢复原始顺序
    loadCustomPrompts()
  }
}

// 移除拖拽相关的监听器

// 事件监听器引用
let unlistenCustomPromptUpdate: (() => void) | null = null
let unlistenWindowMove: (() => void) | null = null

function getInputElement() {
  return textareaRef.value?.textareaElRef || textareaRef.value?.inputElRef || null
}

// 修复输入法候选框位置的函数
function fixIMEPosition() {
  try {
    const inputElement = getInputElement()

    if (inputElement && document.activeElement === inputElement) {
      inputElement.blur()
      window.setTimeout(() => {
        inputElement.focus()
      }, 10)
    }
  }
  catch (error) {
    console.debug('修复IME位置失败:', error)
  }
}

function scheduleIMERepair() {
  if (!textareaRef.value?.isCompositing) {
    return
  }

  const inputElement = getInputElement()
  if (!inputElement || document.activeElement !== inputElement) {
    return
  }

  if (imeRepairTimer) {
    clearTimeout(imeRepairTimer)
  }

  imeRepairTimer = window.setTimeout(() => {
    imeRepairTimer = null
    fixIMEPosition()
  }, 120)
}

// 设置窗口移动监听器
async function setupWindowMoveListener() {
  try {
    const webview = getCurrentWebviewWindow()

    unlistenWindowMove = await webview.onMoved(() => {
      scheduleIMERepair()
    })
  }
  catch (error) {
    console.error('设置窗口移动监听器失败:', error)
  }
}

// 组件挂载时加载自定义prompt
onMounted(async () => {
  await loadCustomPrompts()

  unlistenCustomPromptUpdate = await listen('custom-prompt-updated', () => {
    loadCustomPrompts()
  })

  setupWindowMoveListener()
})

onUnmounted(() => {
  if (unlistenCustomPromptUpdate) {
    unlistenCustomPromptUpdate()
  }

  if (unlistenWindowMove) {
    unlistenWindowMove()
  }

  if (emitUpdateTimer) {
    clearTimeout(emitUpdateTimer)
    emitUpdateTimer = null
  }

  if (dragSortFrame) {
    cancelAnimationFrame(dragSortFrame)
    dragSortFrame = null
  }

  if (focusInputFrame) {
    cancelAnimationFrame(focusInputFrame)
    focusInputFrame = null
  }

  if (imeRepairTimer) {
    clearTimeout(imeRepairTimer)
    imeRepairTimer = null
  }

  destroySortable()
})

// 重置数据
function reset() {
  userInput.value = ''
  selectedOptions.value = []
  uploadedImages.value = []
  emitUpdate()
}

// 更新数据（用于外部同步）
function updateData(data: { userInput?: string, selectedOptions?: string[], draggedImages?: string[] }) {
  if (data.userInput !== undefined) {
    userInput.value = data.userInput
  }
  if (data.selectedOptions !== undefined) {
    selectedOptions.value = data.selectedOptions
  }
  if (data.draggedImages !== undefined) {
    uploadedImages.value = data.draggedImages
  }
}

function getCurrentData() {
  return {
    userInput: userInput.value + generateConditionalContent(),
    selectedOptions: [...selectedOptions.value],
    draggedImages: [...uploadedImages.value],
  }
}

// 移除了文件选择和测试图片功能

// 暴露方法给父组件
defineExpose({
  reset,
  canSubmit,
  statusText,
  updateData,
  handleQuoteMessage,
  getCurrentData,
})
</script>

<template>
  <div class="space-y-3">
    <!-- 预定义选项 -->
    <div v-if="!loading && hasOptions" class="space-y-3" data-guide="predefined-options">
      <h4 class="text-sm font-medium text-white">
        请选择选项
      </h4>
      <n-space vertical size="small">
        <div
          v-for="option in requestOptions"
          :key="option"
          class="rounded-lg p-3 border border-gray-600 bg-gray-100 cursor-pointer hover:opacity-80 transition-opacity"
          @click="handleOptionToggle(option)"
        >
          <n-checkbox
            :value="option"
            :checked="selectedOptions.includes(option)"
            :disabled="submitting"
            size="medium"
            @update:checked="checked => handleOptionChange(option, checked)"
            @click.stop
          >
            {{ option }}
          </n-checkbox>
        </div>
      </n-space>
    </div>

    <!-- 快捷模板固定区 -->
    <div v-if="!loading && hasQuickTemplates" class="space-y-2" data-guide="custom-prompts">
      <div class="text-xs text-on-surface-secondary flex items-center gap-2">
        <div class="i-carbon-bookmark w-3 h-3 text-primary-500" />
        <span>快捷模板 (拖拽调整顺序):</span>
      </div>
      <div class="text-[11px] text-on-surface-secondary/80 leading-relaxed">
        点击模板会直接追加到当前输入；如果输入框为空，则会直接替换为该模板。
      </div>
      <div
        ref="promptContainer"
        data-prompt-container
        class="flex flex-wrap gap-2"
      >
        <div
          v-for="prompt in sortablePrompts"
          :key="prompt.id"
          :title="prompt.description || (prompt.content.trim() ? prompt.content : '清空输入框')"
          class="inline-flex items-center gap-1 px-2 py-1 text-xs bg-container-secondary hover:bg-container-tertiary rounded transition-colors duration-200 select-none border border-gray-600 text-on-surface sortable-item"
        >
          <div class="drag-handle cursor-move p-0.5 rounded hover:bg-container-tertiary transition-colors">
            <div class="i-carbon-drag-horizontal w-3 h-3" />
          </div>
          <div class="inline-flex items-center cursor-pointer" @click="handlePromptClick(prompt)">
            <span>{{ prompt.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 上下文追加工具栏 -->
    <div v-if="!loading && hasContextAppend" class="space-y-2" data-guide="context-append">
      <div class="text-xs text-on-surface-secondary flex items-center gap-2">
        <div class="i-carbon-settings-adjust w-3 h-3 text-primary-500" />
        <span>上下文追加:</span>
      </div>
      <div class="popup-toolbar-list">
        <div
          v-for="prompt in conditionalPrompts"
          :key="prompt.id"
          class="flex items-center justify-between p-2 bg-container-secondary rounded border border-gray-600 hover:bg-container-tertiary transition-colors text-xs"
        >
          <div class="flex-1 min-w-0 mr-2">
            <div class="text-xs text-on-surface truncate font-medium" :title="prompt.condition_text || prompt.name">
              {{ prompt.condition_text || prompt.name }}
            </div>
            <div
              v-if="getConditionalDescription(prompt)"
              class="text-xs text-primary-600 dark:text-primary-400 opacity-50 dark:opacity-60 mt-0.5 truncate leading-tight"
              :title="getConditionalDescription(prompt)"
            >
              {{ getConditionalDescription(prompt) }}
            </div>
          </div>
          <div class="flex-shrink-0">
            <n-switch
              :value="prompt.current_state ?? false"
              size="small"
              @update:value="value => handleConditionalToggle(prompt.id, value)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 图片预览区域 -->
    <div v-if="!loading && uploadedImages.length > 0" class="space-y-3">
      <h4 class="text-sm font-medium text-white">
        已添加的图片 ({{ uploadedImages.length }})
      </h4>

      <!-- 使用 Naive UI 的图片组件，支持预览和放大 -->
      <n-image-group>
        <div class="flex flex-wrap gap-3">
          <div
            v-for="(image, index) in uploadedImages"
            :key="`image-${index}`"
            class="relative"
          >
            <!-- 使用 n-image 组件，启用预览功能 -->
            <n-image
              :src="image"
              width="100"
              height="100"
              object-fit="cover"
              class="rounded-lg border-2 border-gray-300 hover:border-primary-400 transition-colors duration-200 cursor-pointer"
            />

            <!-- 删除按钮 -->
            <n-button
              class="absolute -top-2 -right-2 z-10"
              size="tiny"
              type="error"
              circle
              @click="removeImage(index)"
            >
              <template #icon>
                <div class="i-carbon-close w-3 h-3" />
              </template>
            </n-button>

            <!-- 序号 -->
            <div class="absolute bottom-1 left-1 w-5 h-5 bg-primary-500 text-white text-xs rounded-full flex items-center justify-center font-bold shadow-sm z-5">
              {{ index + 1 }}
            </div>
          </div>
        </div>
      </n-image-group>
    </div>

    <!-- 文本输入区域 -->
    <div v-if="!loading" class="space-y-3">
      <h4 class="text-sm font-medium text-white">
        {{ hasOptions ? '补充说明 (可选)' : '请输入您的回复' }}
      </h4>

      <div v-if="uploadedImages.length === 0" class="text-center">
        <div class="text-xs text-on-surface-secondary">
          💡 提示：可以在输入框中粘贴图片 ({{ pasteShortcut }})
        </div>
      </div>

      <n-input
        ref="textareaRef"
        :value="userInput"
        type="textarea"
        size="small"
        :placeholder="hasOptions ? `您可以在这里添加补充说明... (支持粘贴图片 ${pasteShortcut})` : `请输入您的回复... (支持粘贴图片 ${pasteShortcut})`"
        :disabled="submitting"
        :autosize="{ minRows: 3, maxRows: 4 }"
        data-guide="popup-input"
        @paste="handleImagePaste"
        @update:value="handleUserInputChange"
      />
    </div>
  </div>
</template>

<style scoped>
.popup-toolbar-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

@media (max-width: 767px) {
  .popup-toolbar-list {
    grid-template-columns: 1fr;
  }
}

/* Sortable.js 拖拽样式 */
.sortable-ghost {
  opacity: 0.5;
  transform: scale(0.95);
}

.sortable-chosen {
  cursor: grabbing !important;
}

.sortable-drag {
  opacity: 0.8;
  transform: rotate(5deg);
}
</style>
