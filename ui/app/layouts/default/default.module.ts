import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DefaultComponent } from './default.component';
import { DashboardComponent } from 'ui/app/modules/dashboard/dashboard.component';
import { RouterModule } from '@angular/router';
import { PostsComponent } from 'ui/app/modules/posts/posts.component';
import { SharedModule } from 'ui/app/shared/shared.module';
import {
  NbLayoutModule,
  NbSidebarModule
} from '@nebular/theme'



@NgModule({
  declarations: [
    DefaultComponent,
    DashboardComponent,
    PostsComponent,
  ],
  imports: [
    NbLayoutModule,
    NbSidebarModule,

    CommonModule,
    RouterModule,
    SharedModule
  ]
})
export class DefaultModule { }
