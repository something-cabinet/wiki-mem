import { Injectable } from '@angular/core';
import { toast } from 'ngx-sonner';

@Injectable({ providedIn: 'root' })
export class ToastService {
  success(message: string) { toast.success(message); }
  error(message: string) { toast.error(message); }
  info(message: string) { toast.info(message); }
  warning(message: string) { toast.warning(message); }
}
