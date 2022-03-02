import { ControllerBaseProxy } from 'bizify';
import React from 'react';
import { logService } from '../../services/log.service';
import { ErrorInfo } from '../../typings';

type Data = {
  loaded: boolean;
  log: string;
  errorInfo: ErrorInfo | null;
};

export class LogCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      loaded: false,
      log: '',
      errorInfo: null,
    };
  }

  services = {
    tail: this.$buildService<string>(logService.tail.bind(logService)),
  };

  $dom: React.RefObject<HTMLTextAreaElement> | null = null;

  $timer: any = null;

  init = async (dom: React.RefObject<HTMLTextAreaElement>) => {
    this.$dom = dom;
    this.$timer = setInterval(this.loadContent, 2000);
    await this.loadContent();
    // scroll textarea to bottom
    setTimeout(() => {
      this.scrollToBottom();
    }, 20);
  };

  onDestroy = () => {
    clearInterval(this.$timer);
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
    try {
      this.data.errorInfo = null;
      const log = await this.services.tail.execute();
      this.data.log = log.trimEnd();
    } catch (err: any) {
      this.data.errorInfo = { message: err.message };
    } finally {
      this.data.loaded = true;
    }
  };
}
