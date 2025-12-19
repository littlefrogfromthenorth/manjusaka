<template>
  <PageWrapper dense contentFullHeight contentBackground fixedHeight>
  <div id="terminal-body">
    <div ref="terminalRef" class="terminal" />
  </div>
  </PageWrapper>
</template>

<script lang="ts">
import {
    ref,
    unref,
    computed,
    nextTick,
    reactive,
    watchEffect,
    onMounted,
    onBeforeUnmount, 
    defineComponent,
    watch,
} from 'vue';
import { nps } from './../nps.js';
import '@xterm/xterm/css/xterm.css';
import { PageWrapper } from "/@/components/Page";
import { useGlobSetting } from "/@/hooks/setting";
import { ITheme, Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { downloadByData } from '/@/utils/file/download';
import { copyTextToClipboard } from "/@/hooks/web/useCopyToClipboard";
import { debounce } from 'lodash';
import { useEventListener } from '@vueuse/core';
import { GetApi,PostApi } from "/@/api";
import { useRouter } from 'vue-router';
import { useTabs } from "/@/hooks/web/useTabs";

export default defineComponent({
  name: "AgentCmd",
  components: {
    PageWrapper,
    Terminal, 
    FitAddon, 
    WebLinksAddon
  },
  props: [],
  emits: [],
  setup(props, ctx) {
    const { currentRoute } = useRouter();
    const query = computed(() => unref(currentRoute).params);
    const { closeCurrent, setTitle } = useTabs();

    const terminalRef: any = ref(null);
    const state = reactive({
      agent: {},
      status: 0,
      addon: {
        fit: null as any,
        weblinks: null as any,
      },
    });

    let term: Terminal;
    let socket: WebSocket;
    let pingInterval: any;

    const resize = () => {
      nextTick(() => {
        state.addon.fit.fit();
      });
    };

    const sendMsg = (event: any) => {
      if (state.status === 1) {
        let errmsg = nps.WsMsg.verify(event);
        if (errmsg) {throw Error(errmsg)};
        var uint8 = nps.WsMsg.encode(event).finish();
        socket?.send(uint8);
      }
    };

    const sendResize = (cols: number, rows: number) => {
      let resize = nps.WsResize.create(); 
      resize.cols = cols;
      resize.rows = rows;
      let msg = nps.WsMsg.create();
      msg.resize = resize;
      sendMsg(msg);
    }; 

    function sendPing() {
      let ping = nps.WsPing.create();
      let msg = nps.WsMsg.create();
      msg.ping = ping;
      sendMsg(msg);
    }

    function sendData(key: any) {
      let data = nps.WsData.create();
      data.data = key;
      let msg = nps.WsMsg.create();
      msg.data = data;
      sendMsg(msg);
    }

    function initSocket() {
      const { apiUrl = "/manjusaka" } = useGlobSetting();
      const socketUrl = (window.location.protocol === 'https:' ?"wss:":"ws:") + "//" + window.location.host + apiUrl + "/webproxy/ssh/" + unref(query).id;
      socket = new WebSocket(socketUrl);

      // 监听socket连接
      socket.onopen = () => {
        state.status = 1;
        term.onResize((event) => sendResize(event.cols, event.rows));
        term.onData((event) => sendData(event));
        useEventListener('resize', debounce(resize, 400));
        sendResize(term?.cols, term?.rows);
        pingInterval = setInterval(sendPing, 15000);
      };

      socket.onmessage = (e: Event) => {
        term.write(e.data);
      };

      socket.onclose = (e: CloseEvent) => {
        term.writeln('\r\n\x1b[31m 连接已断开');
        state.status = 0;
        if (pingInterval) clearInterval(pingInterval);
      };

      socket.onerror = (e: Event) => {
        term.writeln('\r\n\x1b[31m 连接错误!');
        state.status = 0;
      };

    }

    function init() {
      term = new Terminal({
        fontSize: 15,
        fontWeight: 'normal',
        fontFamily: 'Consolas', 
        cursorBlink: true,
        disableStdin: false,
        allowProposedApi: true,
        fastScrollModifier: 'ctrl',
        scrollback: 1000000,
        theme: {
          foreground: '#7e9192', // 字体 
          background: '#002833', // 背景色
          cursor: '#268F81', // 设置光标
        } as ITheme,
      });
      term.onKey((e) => {
        if (e.domEvent.ctrlKey || e.domEvent.metaKey){
          if (e.domEvent.key === 'c') {
            copyTextToClipboard(term.getSelection());
            e.domEvent.preventDefault();
          }
          if(e.domEvent.key === 's') {
            const buffer = term.buffer.active;
            let content = '';
            for (let i = 0; i < buffer.length; i++) {
                const line = buffer.getLine(i);
                if (line) {
                    content += line.translateToString(true) + '\n';
                }
            }
            downloadByData(content, state.agent.intranet+'.log');
          }
        }
      });
      term.open(terminalRef.value);

      term.reset();
      term.focus();

      const weblinks = new WebLinksAddon();
      state.addon.weblinks = weblinks;
      term.loadAddon(weblinks);
      resize();

      const fitAddon = new FitAddon();
      state.addon.fit = fitAddon;
      term.loadAddon(fitAddon);
    }
    
    function closeSocket() {
      if (socket && socket.readyState === 1) {
        socket.close();
      }
      if (pingInterval) {
        clearInterval(pingInterval);
      }
    }

    function close() {
      if (term) {
        state.addon.fit.dispose();
        state.addon.weblinks.dispose();
        term.dispose();
      }
    }

    function OnInit() {
      PostApi({
        action: "agentinfo",
        data: { id: unref(query).id },
      }).then((e) => {
          state.agent = e;
          setTitle("终端 - " + state.agent.username + "@" + state.agent.intranet);
      }).catch(() => {
          //createMessage.error("初始化失败");
      });
    }

    onMounted(() => {
      nextTick(() => {
        OnInit();
        init();
        initSocket();
      });
    });

    onBeforeUnmount(() => {
      close();
      closeSocket();
    });

    watchEffect(() => {
      if (props.agent) {
        if (state.agent.id != props.agent.id) {
          state.agent = props.agent;  
        }
      }
    });

    return {
        state,
        terminalRef,
    };
  },
});
</script>

<style lang="less">
  #terminal-body {
    width: 99.9%;
    height: 99.9%;

    .terminal {
      width: 100%;
      height: 100%;
    }
  }
</style>
