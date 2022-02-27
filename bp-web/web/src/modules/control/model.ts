import { ControllerBaseProxy } from 'bizify';
import { serviceService } from '../../services/service.service';
import { ErrorInfo, ServiceInfo } from '../../typings';

type Data = {
  loaded: boolean;
  online: boolean;
  errorInfo: ErrorInfo | null;
  serviceInfo: ServiceInfo | null;
};

export class ControlCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      loaded: false,
      online: false,
      errorInfo: null,
      serviceInfo: null,
    };
  }

  services = {
    query: this.$buildService<{ service_info: ServiceInfo }>(serviceService.query.bind(serviceService)),
    start: this.$buildService<{ service_info: ServiceInfo }>(serviceService.start.bind(serviceService)),
    stop: this.$buildService<void>(serviceService.stop.bind(serviceService)),
  };

  init = async () => {
    try {
      const { service_info } = await this.services.query.execute();
      this.data.serviceInfo = service_info;
      this.data.online = !!service_info;
    } catch (err: any) {
      // console.error(err.status);
    } finally {
      this.data.loaded = true;
    }
  };

  toggleService = async () => {
    try {
      this.data.errorInfo = null;
      if (this.data.online) {
        await this.services.stop.execute();
      } else {
        const { service_info } = await this.services.start.execute();
        this.data.serviceInfo = service_info;
      }
      this.data.online = !this.data.online;
    } catch (err: any) {
      this.data.errorInfo = { message: err.message };
    }
  };
}
