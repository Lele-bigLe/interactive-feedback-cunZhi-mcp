<script setup lang="ts">
import { ref } from 'vue'
import McpToolsTab from '../tabs/McpToolsTab.vue'
import PromptsTab from '../tabs/PromptsTab.vue'
import SettingsTab from '../tabs/SettingsTab.vue'

interface Props {
  currentTheme: string
  alwaysOnTop: boolean
  audioNotificationEnabled: boolean
  audioUrl: string
  windowWidth: number
  windowHeight: number
  fixedWindowSize: boolean
}

interface Emits {
  themeChange: [theme: string]
  toggleAlwaysOnTop: []
  toggleAudioNotification: []
  updateAudioUrl: [url: string]
  testAudio: []
  stopAudio: []
  testAudioError: [error: any]
  updateWindowSize: [size: { width: number, height: number, fixed: boolean }]
  configReloaded: []
}

defineProps<Props>()
const emit = defineEmits<Emits>()

// 处理配置重新加载事件
function handleConfigReloaded() {
  emit('configReloaded')
}

const activeTab = ref('mcp-tools')
</script>

<template>
  <div class="flex flex-col min-h-screen">
    <!-- 主要内容区域 -->
    <div class="flex-1 flex items-start justify-center p-6 pt-6">
      <div class="max-w-6xl w-full">
        <!-- 标题区域 -->
        <div class="mb-5 px-1">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div class="flex items-center gap-3 min-w-0" data-guide="app-logo">
              <div class="w-3 h-3 rounded-full bg-primary-500 flex-shrink-0" />
              <div class="min-w-0">
                <div class="app-panel-title text-xl font-semibold text-on-surface leading-tight">
                  cunzhi
                </div>
                <div class="app-panel-caption mt-1 text-xs text-on-surface-secondary uppercase tracking-[0.22em]">
                  CUNZHI INTERACTION PANEL
                </div>
              </div>
            </div>

            <div class="text-xs text-on-surface-secondary whitespace-nowrap pl-6 sm:pl-0">
              MCP 服务已启动
            </div>
          </div>
        </div>

        <!-- Tab组件 -->
        <n-tabs v-model:value="activeTab" class="main-layout-tabs" type="segment" size="small" justify-content="center" data-guide="tabs">
          <n-tab-pane name="mcp-tools" tab="MCP 工具">
            <McpToolsTab v-if="activeTab === 'mcp-tools'" />
          </n-tab-pane>
          <n-tab-pane name="prompts" tab="参考提示词">
            <PromptsTab v-if="activeTab === 'prompts'" />
          </n-tab-pane>
          <n-tab-pane name="settings" tab="设置" data-guide="settings-tab">
            <SettingsTab
              v-if="activeTab === 'settings'"
              :current-theme="currentTheme"
              :always-on-top="alwaysOnTop"
              :audio-notification-enabled="audioNotificationEnabled"
              :audio-url="audioUrl"
              :window-width="windowWidth"
              :window-height="windowHeight"
              :fixed-window-size="fixedWindowSize"
              @theme-change="$emit('themeChange', $event)"
              @toggle-always-on-top="$emit('toggleAlwaysOnTop')"
              @toggle-audio-notification="$emit('toggleAudioNotification')"
              @update-audio-url="$emit('updateAudioUrl', $event)"
              @test-audio="$emit('testAudio')"
              @stop-audio="$emit('stopAudio')"
              @test-audio-error="$emit('testAudioError', $event)"
              @update-window-size="$emit('updateWindowSize', $event)"
              @config-reloaded="handleConfigReloaded"
            />
          </n-tab-pane>
        </n-tabs>
      </div>
    </div>
  </div>
</template>

<style scoped>
.main-layout-tabs :deep(.n-tabs-nav) {
  margin-bottom: 1.25rem;
}

.main-layout-tabs :deep(.n-tabs-tab) {
  min-width: 132px;
  justify-content: center;
}
</style>
