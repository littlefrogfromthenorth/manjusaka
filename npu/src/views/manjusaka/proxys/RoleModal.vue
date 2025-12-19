<template>
  <BasicModal
    v-bind="$attrs"
    @register="registerModal"
    showFooter
    title="新建隧道代理"
    width="600px"
    @ok="handleSubmit"
  >
    <BasicForm @register="registerForm" />
  </BasicModal>
</template>
<script lang="ts">
import { defineComponent, ref, watch, computed, unref } from "vue";
import { BasicForm, useForm } from "/@/components/Form/index";
import { BasicModal, useModalInner } from "/@/components/Modal";
import { useMessage } from "/@/hooks/web/useMessage";
import { GetApi,PostApi } from "/@/api";

export default defineComponent({
  name: "ProxyModal",
  components: { BasicModal, BasicForm },
  emits: ["success", "register"],
  setup(_, { emit }) {
    const protocol = ref("Socks5");

    const formSchema: FormSchema[] = [
      {
        field: "id",
        label: "NPCID",
        component: "ApiSelect",
        required: true,
        componentProps: {
          api: GetApi,
          params: { action: "agentget" },
          labelField: "id",
          valueField: "id",
          resultField: "items",
        }, 
      },{
        field: "name",
        label: "隧道类型",
        required: true,
        component: "RadioButtonGroup",
        componentProps: {
          options: [
            {
              label: "SOCKS5",
              value: "Socks5"
            },
            {
              label: "端口映射",
              value: "Mapping"
            }
          ],
          onChange: (e) => {
              protocol.value = e;
          },
        },
      },{
        field: "local_port",
        label: "本地端口",
        required: true,
        component: "InputNumber",
      },{
        field: "remote_addr",
        label: "远程地址",
        required: true,
        component: "Input",
        componentProps: {
          placeholder: "127.0.0.1:3389",
        },
        ifShow: () => protocol.value === "Mapping",
      },{
        field: "username",
        label: "用户",
        component: "Input",
        defaultValue: "buut",
        ifShow: () => protocol.value === "Socks5",
      },{
        field: "password",
        label: "密码",
        component: "Input",
        defaultValue: "buut",
        ifShow: () => protocol.value === "Socks5",
      },
    ];

    const [registerForm, { resetFields, setFieldsValue, getFieldsValue, validate }] = useForm({
      labelWidth: 90,
      schemas: formSchema,
      showActionButtonGroup: false,
    });

    const [registerModal, { setModalProps, closeModal }] = useModalInner(async (data) => {
      resetFields();
      setModalProps({ confirmLoading: false });
      setFieldsValue({
        ...data.record,
      });
      protocol.value === "Socks5";
    });


    async function handleSubmit() {
      const values = await validate();
      const { createMessage } = useMessage();
      PostApi({ action: "proxyadd", data: values })
        .then(() => {
            createMessage.success(`成功! 请返回隧道代理列表查看。`);
            emit("success");
            closeModal();
        }).catch((e) => {
            //createMessage.error(`失败,查看是否加载npc2或者是端口被占用。${e} `);
        }).finally(() => {
        });
    }

    return {
      registerModal,
      registerForm,
      handleSubmit,
    };
  },
});
</script>
