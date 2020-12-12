import _ from './index.html';
import React from 'react';
import ReactDOM from 'react-dom';
import App from './App/App.jsx';
import {BrowserRouter, Switch, Route} from "react-router-dom";

ReactDOM.render((
    <BrowserRouter>
        <Switch>
            <Route component={App} />
        </Switch>
    </BrowserRouter>
), document.getElementById('app'));
