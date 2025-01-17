import {Observable, ReplaySubject, Subject} from 'rxjs'
import {
    ColInfo,
    DisplayPatch,
    SheetBlocks,
    SheetColInfo,
    SheetComments,
    SheetMergeCells,
    SheetNames,
    SheetRowInfo,
    SheetStyles,
    SheetValues,
} from '@/bindings'
import {ClientSend, ServerSend} from '@/message'
import {Payload, PayloadsTransaction, adaptTransaction} from '@/api'
import {hasOwnProperty} from '@/core'
import {SheetService} from '@/core/data/sheet'
import {Service as StandAloneService} from '@/wasm_svc/service'
import {injectable, inject} from 'inversify'
import {TYPES} from '@/core/ioc/types'
import {getID} from '@/core/ioc/id'

@injectable()
export class Backend {
    readonly id = getID()
    constructor(@inject(TYPES.Sheet) private readonly sheetSvc: SheetService) {
        this._wasmSvc.output$.subscribe((e) => {
            this.handleResponse(e)
        })
    }
    get render$(): Observable<void> {
        return this._render$
    }

    get sheetUpdated$(): Observable<readonly number[]> {
        return this._sheetUpdated$
    }
    send$ = new ReplaySubject<Blob>(5)
    handleResponse(msg: ServerSend) {
        this._handleServerSend(msg)
    }
    /**
     * Send payloads to backend for transaction.
     * @param undoable Default true
     */
    sendTransaction(payloads: Payload[], undoable = true) {
        if (payloads.length === 0) return
        const t = new PayloadsTransaction(payloads, undoable)
        const msg = adaptTransaction(t)
        this.send({$case: 'transaction', transaction: msg})
    }

    send(msg: ClientSend) {
        this._wasmSvc.input$.next(msg)
        return
    }
    private _render$ = new Subject<void>()
    // Sever tolds the client that these sheets are dirty.
    private _sheetUpdated$ = new Subject<readonly number[]>()
    private _wasmSvc = new StandAloneService([])
    private _handleServerSend(serverSend: ServerSend) {
        if (serverSend.$case === 'displayResponse') {
            const e = serverSend.displayResponse
            this.sheetSvc.clear()
            e.patches.forEach((p) => {
                this._handleDisplayArea(p)
            })
            this._render$.next()
        } else if (serverSend.$case === 'actionEffect') {
            const sheetUpdated = serverSend.actionEffect.sheets
            this._sheetUpdated$.next(sheetUpdated)
        }
    }

    private _handleDisplayArea(patches: DisplayPatch) {
        const displayArea = patches
        if (hasOwnProperty(displayArea, 'values')) {
            const values = displayArea.values as SheetValues
            const sheet = values.sheetIdx
            values.values.forEach((v) => {
                const {row, col, formula, value} = v
                this.sheetSvc.setCell(row, col, sheet, {value, formula})
            })
        } else if (hasOwnProperty(displayArea, 'styles')) {
            const styles = displayArea.styles as SheetStyles
            const sheet = styles.sheetIdx
            styles.styles.forEach((s) => {
                const {row, col, style} = s
                if (style) this.sheetSvc.setCell(row, col, sheet, {style})
            })
        } else if (hasOwnProperty(displayArea, 'rowInfo')) {
            const rowInfo = displayArea.rowInfo as SheetRowInfo
            const sheet = rowInfo.sheetIdx
            rowInfo.info.forEach((i) => {
                this.sheetSvc.setRowInfo(i.idx, i, sheet)
            })
        } else if (hasOwnProperty(displayArea, 'colInfo')) {
            const colInfo = displayArea.colInfo as SheetColInfo
            const sheet = colInfo.sheetIdx
            colInfo.info.forEach((i) => {
                this.sheetSvc.setColInfo(i.idx, i, sheet)
            })
        } else if (hasOwnProperty(displayArea, 'sheetNames')) {
            const sheetNames = displayArea.sheetNames as SheetNames
            sheetNames.names.forEach((name, i) => {
                this.sheetSvc.setSheet(i, {name})
            })
        } else if (hasOwnProperty(displayArea, 'mergeCells')) {
            const merges = displayArea.mergeCells as SheetMergeCells
            const cells = merges.mergeCells
            this.sheetSvc.setSheet(merges.sheetIdx, {
                merges: cells,
            })
        } else if (hasOwnProperty(displayArea, 'comments')) {
            const comments = displayArea.comments as SheetComments
            const sheet = comments.sheetIdx
            this.sheetSvc.setSheet(sheet, {
                comments: comments.comments,
            })
        } else if (hasOwnProperty(displayArea, 'blocks')) {
            const {sheetIdx, blocks} = displayArea.blocks as SheetBlocks
            this.sheetSvc.setBlocks(sheetIdx, blocks)
        }
    }
}
