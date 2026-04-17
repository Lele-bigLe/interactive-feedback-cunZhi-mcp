<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { computed, onMounted, ref } from 'vue'
import { useMcpToolsReactive } from '../../composables/useMcpTools'

const { mcpTools, loading, loadMcpTools } = useMcpToolsReactive()

const message = useMessage()
const showToolConfigModal = ref(false)
const zhiConfig = ref({
  request_timeout_minutes: 10,
})

const zhiTool = computed(() => mcpTools.value.find(tool => tool.id === 'zhi') ?? null)

async function loadZhiConfig() {
  try {
    const config = await invoke('get_zhi_tool_config') as {
      request_timeout_ms: number
    }

    zhiConfig.value = {
      request_timeout_minutes: Math.max(1, Math.round((config.request_timeout_ms || 600000) / 60000)),
    }
  }
  catch (err) {
    message.error(`加载 cunzhi 配置失败: ${err}`)
  }
}

async function openToolConfig() {
  await loadZhiConfig()
  showToolConfigModal.value = true
}

async function saveZhiConfig() {
  try {
    const timeoutMinutes = Math.max(1, Math.round(zhiConfig.value.request_timeout_minutes || 10))
    await invoke('set_zhi_tool_config', {
      requestTimeoutMs: timeoutMinutes * 60 * 1000,
    })
    zhiConfig.value.request_timeout_minutes = timeoutMinutes
    message.success('cunzhi 倒计时配置已保存')
    showToolConfigModal.value = false
  }
  catch (err) {
    message.error(`保存 cunzhi 配置失败: ${err}`)
  }
}

onMounted(async () => {
  try {
    await loadMcpTools()
  }
  catch (err) {
    message.error(`加载MCP工具配置失败: ${err}`)
  }
})
</script>

<template>
  <div class="max-w-3xl mx-auto tab-content">
    <n-space vertical size="large">
      <n-alert type="info" title="工具裁剪已生效">
        <template #icon>
          <div class="i-carbon-checkmark-outline text-lg" />
        </template>
        非核心的记忆管理和代码搜索集成已从当前交互面板中移除，当前仅保留核心的 cunzhi 交互确认工具。
      </n-alert>

      <div v-if="loading" class="text-center py-8">
        <n-spin size="medium" />
        <div class="mt-2 text-sm opacity-60">
          加载MCP工具配置中...
        </div>
      </div>

      <template v-else-if="zhiTool">
        <n-card size="small" class="shadow-sm hover:shadow-md transition-shadow duration-200">
          <template #header>
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <div
                  class="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0"
                  :class="[zhiTool.icon_bg, zhiTool.dark_icon_bg]"
                >
                  <div :class="zhiTool.icon" />
                </div>

                <div class="flex-1 min-w-0">
                  <n-space align="center">
                    <div class="text-lg font-medium tracking-tight">
                      {{ zhiTool.name }}
                    </div>
                    <n-tag type="info" size="small" :bordered="false">
                      必需
                    </n-tag>
                    <n-tag type="success" size="small" :bordered="false">
                      已启用
                    </n-tag>
                  </n-space>
                  <div class="text-sm opacity-60 font-normal mt-1">
                    {{ zhiTool.description }}
                  </div>
                </div>
              </div>

              <n-button size="small" quaternary circle @click="openToolConfig">
                <template #icon>
                  <div class="i-carbon-settings-adjust w-4 h-4" />
                </template>
              </n-button>
            </div>
          </template>

          <n-space vertical size="medium">
            <div class="flex items-center text-sm leading-relaxed">
              <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-3 flex-shrink-0" />
              <span class="opacity-90">当前只保留 1 个核心 MCP 工具，主界面负担更轻，配置项也更集中。</span>
            </div>
            <div class="flex items-center text-sm leading-relaxed">
              <div class="w-1.5 h-1.5 bg-blue-500 rounded-full mr-3 flex-shrink-0" />
              <span class="opacity-90">点击右上角齿轮可调整默认倒计时，用于新的 cunzhi 请求。</span>
            </div>
          </n-space>
        </n-card>
      </template>

      <n-empty v-else description="当前没有可用的 MCP 工具配置" />
    </n-space>

    <n-modal
      v-model:show="showToolConfigModal"
      preset="card"
      :closable="true"
      :mask-closable="true"
      title="cunzhi 工具配置"
      style="width: 720px"
      :bordered="false"
      size="huge"
    >
      <n-space vertical size="large">
        <n-alert type="info" title="cunzhi 倒计时配置">
          <template #icon>
            <div class="i-carbon-timer text-lg" />
          </template>
          默认倒计时为 10 分钟。到期后若用户没有反应会自动重新发起；同一项目在倒计时未结束前不可重复发起。
        </n-alert>

        <n-card size="small">
          <template #header>
            <div class="font-medium">
              倒计时设置
            </div>
          </template>

          <n-space vertical size="large">
            <n-form-item label="默认倒计时（分钟）">
              <n-input-number
                v-model:value="zhiConfig.request_timeout_minutes"
                :min="1"
                :max="60"
                :precision="0"
                placeholder="10"
              />
              <template #feedback>
                作用于新的 cunzhi 请求。允许范围 1 - 60 分钟，默认 10 分钟。
              </template>
            </n-form-item>

            <n-alert type="warning" title="生效说明">
              <template #icon>
                <div class="i-carbon-warning" />
              </template>
              修改后会写入本地配置文件，新发起的 cunzhi 请求会使用新的倒计时时间。
            </n-alert>
          </n-space>
        </n-card>
      </n-space>

      <template #footer>
        <n-space justify="end">
          <n-button @click="showToolConfigModal = false">
            取消
          </n-button>
          <n-button type="primary" @click="saveZhiConfig">
            保存配置
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>
