import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { DefaultModule } from './layouts/default/default.module';
import { 
  NbSidebarModule, 
  NbLayoutModule 
} from '@nebular/theme';

@NgModule({
  declarations: [
    AppComponent,
  ],
  imports: [
    NbLayoutModule,
    NbSidebarModule.forRoot(),

    BrowserModule,
    AppRoutingModule,
    DefaultModule,
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
