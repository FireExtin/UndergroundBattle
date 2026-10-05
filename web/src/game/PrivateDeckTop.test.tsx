import {fireEvent,render,screen,within} from '@testing-library/react';
import {expect,it,vi} from 'vitest';
import {Table} from './Table';
import {testCard,testCatalog,testView} from './testFixtures';
import type {Card,View} from './types';
const top:Card={...testCard,instanceId:'top-private',cardId:'LC01',name:'西比尔',text:'仅本座位收到的当前顶牌',owner:'p0',controller:'p0'};
const view:View={...testView,status:'playing',hand:[],legalActions:[],privateDeckTop:top};
const props={catalog:testCatalog,busy:false,onAction:vi.fn()};
it('opens only the current private projection in the existing reader and drops it when permission is lost',()=>{
 const {rerender}=render(<Table {...props} view={view}/>);
 expect(screen.queryByText(top.text!)).not.toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:'检视顶牌'}));
 expect(within(screen.getByRole('dialog')).getByText(top.text!)).toBeInTheDocument();
 expect(within(screen.getByRole('dialog')).getByText('你的牌库顶牌 · 仅你可见')).toBeInTheDocument();
 rerender(<Table {...props} view={{...view,version:2,privateDeckTop:undefined,hand:[top]}}/>);
 expect(screen.queryByRole('dialog')).not.toBeInTheDocument();expect(screen.queryByRole('button',{name:'检视顶牌'})).not.toBeInTheDocument();
});
it('invalidates a reader on a changed top, even when the previous instance is now in the authorized hand',()=>{
 const {rerender}=render(<Table {...props} view={view}/>);fireEvent.click(screen.getByRole('button',{name:'检视顶牌'}));
 const next={...top,instanceId:'next-private',name:'新的顶牌',text:'最新顶牌文字'};
 rerender(<Table {...props} view={{...view,version:2,privateDeckTop:next,hand:[top]}}/>);
 expect(screen.queryByRole('dialog')).not.toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'检视顶牌'}));
 expect(within(screen.getByRole('dialog')).getByText(next.text)).toBeInTheDocument();expect(within(screen.getByRole('dialog')).queryByText(top.text!)).not.toBeInTheDocument();
});
it('clears private reading on room or viewer switches and omits the control for old projections or an empty deck',()=>{
 const {rerender}=render(<Table {...props} view={view}/>);fireEvent.click(screen.getByRole('button',{name:'检视顶牌'}));
 rerender(<Table {...props} view={{...view,you:'p1',privateDeckTop:undefined}}/>);expect(screen.queryByRole('dialog')).not.toBeInTheDocument();expect(screen.queryByText(top.text!)).not.toBeInTheDocument();
 rerender(<Table {...props} view={view}/>);fireEvent.click(screen.getByRole('button',{name:'检视顶牌'}));
 rerender(<Table {...props} view={{...view,roomId:'new-room'}}/>);expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
 rerender(<Table {...props} view={{...view,privateDeckTop:undefined}}/>);expect(screen.queryByRole('button',{name:'检视顶牌'})).not.toBeInTheDocument();
});
