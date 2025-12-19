<template>
  <div>
    <BasicModal
      v-bind="$attrs"
      :width="600"
      @register="registerModal"
      title="运行插件"
      okText="下发指令"
      @ok="handleSubmit">
      <BasicForm @register="registerForm" />
    </BasicModal>
  </div>
</template>
<script lang="ts">
import { defineComponent, watch } from "vue";
import { BasicModal, useModalInner } from "/@/components/Modal";
import { BasicForm, useForm } from "/@/components/Form/index";
import { nps } from './nps.js';

export default defineComponent({
  name: "RoleModal",
  components: {
    BasicModal,
    BasicForm,
  },
  emits: ["success"],
  setup(props, ctx) {
    const [registerForm, { setFieldsValue, getFieldsValue }] = useForm({
      labelWidth: 120,
      schemas: [
        {
          field: "name",
          component: "ApiSelect",
          label: "插件名称",
          componentProps: {
            value: { name: "", args: "", exe: Boolean },
            api: PostApi,
            params: { action: "plugselectreq" },
            labelField: "fullname",
            valueField: "name",
            resultField: "items",
            onChange: (name, e) => {
              setFieldsValue(e);
            },
          },
        },
        {
          field: "args",
          component: "InputTextArea",
          label: "运行参数",
        },
        {
          field: "exe",
          component: "Checkbox",
          label: "可执行文件？",
        },
      ],
      showResetButton: false,
      showSubmitButton: false,
    });

    const registerModal = useModalInner(async (data) => {
      setFieldsValue(data);
    });

    function handleSubmit() {
      let data = getFieldsValue();
      let event = nps.AgentEvent.create();
      let msg = nps.PluginExec.create();
      msg.name = data.name;
      msg.args = data.args;
      msg.plugtype = data.exe ? "exe" : "";
      event.pluginexec = msg;
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
