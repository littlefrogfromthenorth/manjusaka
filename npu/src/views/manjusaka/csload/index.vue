<template>
  <PageWrapper title="生成器" contentFullHeight contentBackground fixedHeight>
    <CollapseContainer>
      <template #title>
        <a-button click="GetPass" type="primary"> 生成附件 </a-button>
        <a-button type="ghost" />
        <a-button type="ghost" />
      </template>
      <BasicForm @register="registerForm" />
    </CollapseContainer>
  </PageWrapper>
</template>


<script lang="ts">
import { defineComponent, h, reactive } from "vue";
import { BasicForm, useForm } from "/@/components/Form/index";
import { PageWrapper } from "/@/components/Page";
import { CollapseContainer } from "/@/components/Container/index";
import { Alert } from "ant-design-vue";
import { UploadApi,PostApi } from "/@/api";
import { useMessage } from "/@/hooks/web/useMessage";
import { useDrawer } from "/@/components/Drawer";
import { useUserStore } from "/@/store/modules/user";
import RoleDrawer from "./RoleDrawer.vue";
import { Switch, Radio ,Button, Input } from "ant-design-vue";

export default defineComponent({
  name: "Csload",
  components: {
    BasicForm,
    RoleDrawer,
    PageWrapper,
    Alert,
    CollapseContainer,
    Input,
  },
  setup() {
    const state = reactive({
      url: "",
      email: {},
      target: {},
    });
    const { createMessage } = useMessage();
    const userStore = useUserStore();
    const [registerDrawer, { openDrawer }] = useDrawer();

    const [registerForm, { setFieldsValue, getFieldsValue }] = useForm({
      labelWidth: 120,
      schemas: [
        {
          field: "proj",
          component: "Input",
          label: "项目目标",
        },
        {
          field: "arch",
          component: "Input",
          label: "平台",
          component: "RadioButtonGroup",
          defaultValue: "x86_64",
          componentProps: (form) => ({
            options: [
              { label: "x86_64", value: "x86_64" },
              { label: "aarch64", value: "aarch64" },
            ],
          }),
        },
        {
          field: "platform",
          component: "Input",
          label: "类型",
          component: "RadioButtonGroup",
          defaultValue: "windows",
          componentProps: (form) => ({
            options: [
              { label: "windows", value: "windows" },
              { label: "linux", value: "linux" },
              { label: "macos", value: "macos" },
            ],
          }),
        },
        {
          field: "authrun",
          component: "Input",
          label: "维权方式",
          component: "RadioButtonGroup",
          defaultValue: "service",
          componentProps: (form) => ({
            options: [
              { label: "注册服务", value: "service" },
              { label: "计划任务", value: "taskschd" },
              { label: "不自启", value: "none" },
            ],
          }),
        },
        {
          field: 'field23',
          component: 'Upload',
          label: '图标',
          defaultValue: [
            'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png',
          ],
          componentProps: {
            api: UploadApi,
            accept: ['png', 'jpeg', 'jpg'],
            maxSize: 1,
            maxNumber: 1,
          },
        },
        {
          field: 'field23',
          component: 'Upload',
          label: '证书',
          defaultValue: [
            'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png',
          ],
          componentProps: {
            api: UploadApi,
            accept: ['png', 'jpeg', 'jpg'],
            maxSize: 1,
            maxNumber: 1,
          },
        }
      ],
      showResetButton: false,
      showSubmitButton: false,
    });
     
    function AgentCreate() {

    }
    function handleSuccess() {
      
    }



    return {
      registerForm,
      registerDrawer,
      handleSuccess,
      AgentCreate,
      state
    };
  },
});
</script>


