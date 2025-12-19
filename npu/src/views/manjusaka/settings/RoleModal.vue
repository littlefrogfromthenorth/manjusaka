<template>
  <BasicModal 
    v-bind="$attrs" 
    title="修改密码" 
    @register="registerModal" 
    @ok="handleSubmit">
    <BasicForm @register="registerForm" />
  </BasicModal>
</template>
<script lang="ts">
import { defineComponent, ref, h, computed, unref } from "vue";
import { BasicModal, useModalInner } from "/@/components/Modal";
import { BasicForm, useForm } from "/@/components/Form/index";
import { PostApi } from "/@/api";
import { useMessage } from "/@/hooks/web/useMessage";
import { encryptBySha256 } from "/@/utils/cipher";
import { useUserStore } from "/@/store/modules/user";
import { Input, Col, Row, Image } from "ant-design-vue";

export default defineComponent({
  name: "RoleModal",
  components: { BasicModal, BasicForm },
  emits: ["success", "register"],
  setup(_, { emit }) {
    const confirmLoading = ref(false);
    const { createMessage } = useMessage();
    const userStore = useUserStore();
    const userinfo = computed(() => {
      return userStore.getUserInfo || {};
    });
    const captchaImage = ref('');
    const captchaImageUrl = ref('../captcha');
    const refreshCaptcha = () => {
      captchaImageUrl.value = `../captcha?t=${new Date().getTime()}`;
    };

    const [registerForm, { setFieldsValue, updateSchema, resetFields, validate, getFieldsValue  }] = useForm({
      labelWidth: 60,
      schemas: [
        {
          field: "username",
          label: "用户名",
          component: "Input",
          defaultValue: unref(userinfo)?.username || '',
          componentProps: {
            disabled: true,
          },
        },
        {
          field: "password",
          label: "旧密码",
          required: true,
          component: "InputPassword",
          componentProps: {
            placeholder: "请输入当前密码",
          },
        },
        {
          field: "newpass",
          label: "新密码",
          required: true,
          component: "InputPassword",
          componentProps: {
            placeholder: "请输入新密码",
          }
        },
        {
          field: "captcha",
          label: "验证码",
          component: "Input",
          required: true,
          renderColContent: ({ model }) => {
            return h(Row, { gutter: 10 }, [ 
              h(Col, { span: 3, style: { textAlign: "right" } }, "验证码"),
              h(Col, { span: 16 }, [ // 输入框占 16 列
                h(Input, {
                  placeholder: "请输入验证码",
                  onChange: (e: Event) => {
                    captchaImage.value =  (e.target as HTMLInputElement).value;
                  }
                })
              ]),
              h(Col, { span: 5 }, [ 
                h(Image, {
                  src: captchaImageUrl.value,
                  alt: "验证码，点击刷新",
                  width: "100%",
                  height: "32px",
                  onClick: refreshCaptcha, 
                  preview: false // 禁用预览功能
                })
              ])
            ]);
          }
        }
      ],
      showActionButtonGroup: false,
      actionColOptions: {
        span: 23,
      },
    });

    const [registerModal, { closeModal }] = useModalInner(async (data) => {
      resetFields();
      setFieldsValue({
        username: unref(userinfo)?.username || ''
      });
    });

    const getFormModel = getFieldsValue;

    async function handleSubmit() {
      confirmLoading.value = true;

      const values = await validate();

      const submitData = {
        ...values,
        password: encryptBySha256(values.password),
        newpass: encryptBySha256(values.newpass),
        captcha: captchaImage.value,
      };

      PostApi({ action: "password", data: submitData })
      .then((e) => {
        if (e==="ok"){
          createMessage.success("密码修改成功");
          resetFields();
          closeModal();
          userStore.logout(true);
        }else{
          createMessage.error(`${e}`);
        }
      }).catch(() => {
        //createMessage.error("失败 ");
      }).finally(() => {
        confirmLoading.value = false;
        refreshCaptcha();
      });
    }

    return { registerModal, registerForm, userinfo, confirmLoading, handleSubmit};
  },
});
</script>
