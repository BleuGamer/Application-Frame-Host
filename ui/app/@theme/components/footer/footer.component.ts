import { Component } from '@angular/core';

@Component({
  selector: 'ngx-footer',
  styleUrls: ['./footer.component.scss'],
  template: `
    <span class="created-by">DEVELOPMENT STANDIN</span>
    <div class="socials">
      <a href="https://github.com/BleuGamer/Application-Frame-Host" target="_blank" class="ion ion-social-github"></a>
    </div>
  `,
})
export class FooterComponent {
}
