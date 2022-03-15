import { ControllerBaseProxy } from 'bizify';
import React from 'react';
import { logService } from '../../services/log.service';

type Data = {
  log: string;
  autoRefresh: boolean;
};

export class LogCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      log: '',
      autoRefresh: false,
    };
  }

  services = {
    tail: this.$buildService<string>(logService.tail.bind(logService)),
  };

  $dom: React.RefObject<HTMLTextAreaElement> | null = null;

  $timer: any = null;

  init = async (dom: React.RefObject<HTMLTextAreaElement>) => {
    this.$dom = dom;
    await this.loadContent();

    setTimeout(() => {
      this.scrollToBottom();
    }, 20);
  };

  onDestroy = () => {
    this.stopPolling();
  };

  handleAutoRefreshClick = () => {
    const autoRefresh = !this.data.autoRefresh;

    if (autoRefresh) {
      this.startPolling();
    } else {
      this.stopPolling();
    }

    this.data.autoRefresh = autoRefresh;
  };

  handleRefresh = async () => {
    await this.loadContent();
    this.scrollToBottom();
  };

  private scrollToBottom = () => {
    const dom = this.$dom?.current;

    if (dom) {
      dom.scrollTop = dom.scrollHeight;
    }
  };

  private loadContent = async () => {
    this.data.log = await this.services.tail.execute();
  };

  private startPolling = () => {
    this.$timer = setInterval(this.loadContent, 2000);
  };

  private stopPolling = () => {
    clearInterval(this.$timer);
  };
}
