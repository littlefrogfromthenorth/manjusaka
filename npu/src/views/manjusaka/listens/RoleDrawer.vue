<template>
  <BasicDrawer
    v-bind="$attrs"
    @register="registerDrawer"
    showFooter
    :title="getTitle"
    width="800px"
    @ok="handleSubmit"
  >
    <BasicForm @register="registerForm" />
  </BasicDrawer>
</template>
<script lang="ts">
import { defineComponent, ref, computed, unref } from "vue";
import { BasicForm, useForm } from "/@/components/Form/index";
import { BasicDrawer, useDrawerInner } from "/@/components/Drawer";
import { useMessage } from "/@/hooks/web/useMessage";
import { PostApi } from "/@/api";

export default defineComponent({
  name: "RoleDrawer",
  components: { BasicDrawer, BasicForm },
  emits: ["success", "register"],
  setup(_, { emit }) {
    const isUpdate = ref(true);

    const formSchema: FormSchema[] = [
      {
        field: "id",
        label: "监听器ID",
        component: "Input",
        componentProps: {
          disabled: true,
        },
      },{
        field: "transport",
        label: "协议",
        required: true,
        component: "RadioButtonGroup",
        componentProps: {
          options: [
            {
              label: "TCP",
              value: "tcp"
            },
            {
              label: "KCP",
              value: "kcp"
            },
            {
              label: "WebSocket",
              value: "websocket"
            },
            {
              label: "Forward(tcp)",
              value: "forward"
            }
          ]
        },
      },{
        field: "listenaddr",
        label: "监听地址",
        required: true,
        component: "Input",
        defaultValue: "0.0.0.0:32000",
      },{
        field: "onlineaddr",
        label: "上线地址",
        component: "Input",
        required: true,
        componentProps: {
          placeholder: "127.0.0.1:32000",
        },
      },{
        field: "RTT",
        label: "RTT",
        component: "RadioButtonGroup",
        componentProps: {
          options: [
            {
              label: "KK",value: "KK"
            },
            {
              label: "NN",value: "NN"
            },
            {
              label: "XX",value: "XX"
            },
            {
              label: "NK",value: "NK"
            },
            {
              label: "NX",value: "NX"
            },
            {
              label: "IK",value: "IK"
            },
          ]
        },
      },{
        field: "DH",
        label: "DH",
        component: "RadioButtonGroup",
        componentProps: {
          options: [
            {
              label: "25519",value: "25519"
            },
            {
              label: "448",value: "448"
            }
          ]
        },
      },{
        field: "ENC",
        label: "ENC ",
        component: "RadioButtonGroup",
        componentProps: {
          options: [
            {
              label: "ChaChaPoly",value: "ChaChaPoly"
            },
            {
              label: "AESGCM",value: "AESGCM"
            },
          ]
        },
      },{
        field: "HASH",
        label: "HASH ",
        component: "RadioButtonGroup",
        componentProps: {
          options: [
            {
              label: "BLAKE2s",value: "BLAKE2s"
            },
            {
              label: "BLAKE2b",value: "BLAKE2b"
            },
            {
              label: "SHA256",value: "SHA256"
            },
            {
              label: "SHA512",value: "SHA512"
            },
          ]
        },
      },{
        field: "proxyaddr",
        label: "代理地址",
        component: "Input",
        componentProps: {
          placeholder: "(socks|http|https)://user:pass@127.0.0.1:8080",
        },
      },{
        field: "enckey",
        label: "加密密钥",
        component: "Input",
        defaultValue: "manjusaka",
      },
    ];

    const [registerForm, { resetFields, setFieldsValue, validate }] = useForm({
      labelWidth: 90,
      schemas: formSchema,
      showActionButtonGroup: false,
    });

    const [registerDrawer, { setDrawerProps, closeDrawer }] = useDrawerInner(async (data) => {
      resetFields();
      setDrawerProps({ confirmLoading: false });
      isUpdate.value = !!data?.isUpdate;
      if (unref(isUpdate)) {
        setFieldsValue({
          ...data.record,
          transport: data.record.transport || "tcp"
        });
      }else {
        setFieldsValue({
          transport: "tcp"
        });
      }
    });

    const getTitle = computed(() => (!unref(isUpdate) ? "新增" : "编辑"));

    async function handleSubmit() {
      try {
        const values = await validate();
        const { createMessage } = useMessage();
        setDrawerProps({ confirmLoading: true });
        PostApi({ action: "listenupdate", data: values })
          .then(() => {
            createMessage.success(`成功! 请刷新列表`);
          })
          .catch(() => {
            //createMessage.error("失败 ");
          })
          .finally(() => {
            emit("success");
          });
        closeDrawer();
      } finally {
        setDrawerProps({ confirmLoading: false });
      }
    }

    return {
      registerDrawer,
      registerForm,
      getTitle,
      handleSubmit,
    };
  },
});
</script>
