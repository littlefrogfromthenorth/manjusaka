<template>
    <BasicModal
      v-bind="$attrs"
      :width="600"
      @register="registerModal"
      title="运行插件"
      okText="下发指令"
      @ok="handleSubmit">
      <BasicForm @register="registerForm" />
    </BasicModal>
</template>
<script lang="ts">
import { defineComponent, watch } from "vue";
import { BasicModal, useModalInner } from "/@/components/Modal";
import { BasicForm, useForm } from "/@/components/Form/index";
import { nps } from './nps.js';
import { PostApi } from "/@/api";

export default defineComponent({
  name: "PlugModal",
  components: {
    BasicModal,
    BasicForm,
  },
  emits: ["success"],
  setup(props, ctx) {
    const [registerForm, { setFieldsValue, getFieldsValue }] = useForm({
      labelWidth: 100,
      schemas: [{
        required: true,
        field: "name",
        component: "ApiSelect",
        label: "插件名称",
        componentProps: {
            api: PostApi,
            params: { action: "pluglist" },
            labelField: "name",
            valueField: "name",
            resultField: "items",
            onChange: (name, e) => {
              setFieldsValue(e);
            },
          },
        },
        {
          field: "entry",
          component: "Input",
          label: "入口点",
        },
        {
          field: "args",
          component: "InputTextArea",
          label: "运行参数",
          componentProps: {
            autoSize: true,
          },
        },
      ],
      showResetButton: false,
      showSubmitButton: false,
    });

    const registerModal = useModalInner(async (data) => {
      setFieldsValue(data);
    });

    function getArch(fileName: string): string {
      const ext = fileName.includes('.') 
        ? fileName.split('.').pop()!.toLowerCase() 
        : '';
      return ext;
    }

    function handleSubmit() {
      let data = getFieldsValue();
      let event = nps.AgentEvent.create();
      let plug = nps.Plugin.create();
      plug.act = "ex";
      plug.name = data.name.split('.')[0];
      plug.entry = data.entry;
      plug.args = data.args;
      plug.arch = getArch(data.name)
      event.plugin = plug;
      ctx.emit("success", event);
    }

    return {
      registerModal,
      registerForm,
      handleSubmit,
    };
  },
});
</script>
