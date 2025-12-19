<template>
  <BasicDrawer
    v-bind="$attrs"
    @register="registerDrawer"
    showFooter
    :title="getTitle"
    width="600px"
    @ok="handleSubmit"
  >
    <BasicForm @register="registerForm" />
  </BasicDrawer>
</template>
<script lang="ts">
import { defineComponent, ref, reactive, computed, unref } from "vue";
import { BasicForm, useForm } from "/@/components/Form/index";
import { BasicDrawer, useDrawerInner } from "/@/components/Drawer";
import { useMessage } from "/@/hooks/web/useMessage";
import { PostApi } from "/@/api";

export default defineComponent({
  name: "RoleDrawer",
  components: { BasicDrawer, BasicForm },
  emits: ["success", "register"],
  setup(_, { emit }) {
    const state = reactive({
      listens: [],
    });
    const isUpdate = ref(true);

    const formSchema: FormSchema[] = [
      {
        field: "id",
        label: "项目ID",
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
              label: "HTTP(s)",
              value: "http"
            },
            {
              label: "OSS",
              value: "oss"
            },
            {
              label: "DNS",
              value: "dns"
            },
          ]
        },
      },
      {
        field: "name",
        label: "项目名称",
        required: true,
        component: "Input",
      },
      {
        field: "callback1",
        label: "回调地址",
        required: true,
        component: "Input",
        componentProps: {
          placeholder: "http://192.168.93.1:31000",
        },
      },
      {
        field: "listen",
        label: "连接地址",
        component: "ApiSelect",
        componentProps: {
          api: PostApi,
          params: { action: "listenlist" },
          labelField: "onlineaddr",
          valueField: "id",
          resultField: "items",
        },
      },
      {
        field: "domain",
        label: "上线域名",
        component: "Input",
        componentProps: {
          placeholder: "域前置",
        },
      },
      {
        field: "proxy",
        label: "代理地址",
        component: "Input",
        componentProps: {
          placeholder: "(socks|http|https)://user:pass@127.0.0.1:8080",
        },
      },
      {
        field: "route",
        label: "路由",
        component: "Input",
        componentProps: {
          //disabled: true,
        },
      },
      {
        field: "enckey",
        label: "密钥",
        component: "Input",
        componentProps: {
          //disabled: true,
        },
      },
      {
        field: "config",
        label: "其它配置",
        component: "InputTextArea",
        componentProps: {
          autoSize: true,
        },
      },
      {
        field: "note",
        label: "备注",
        component: "Input",
      },
    ];

    const [registerForm, { resetFields, setFieldsValue, validate }] = useForm({
      labelWidth: 100,
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
          transport: data.record.transport || "http"
        });
      }else {
        setFieldsValue({
          transport: "http"
        });
      }
    });

    const getTitle = computed(() => (!unref(isUpdate) ? "新增项目" : "编辑项目"));

    async function handleSubmit() {
      try {
        const values = await validate();
        const { createMessage } = useMessage();
        setDrawerProps({ confirmLoading: true });
        PostApi({ action: "projectupdate", data: values })
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
      state,
      registerDrawer,
      registerForm,
      getTitle,
      handleSubmit,
    };
  },
});
</script>
